use crate::args::Fuzz;
use crate::error_exit;
use crate::executor::{deploy_resource, get_client};
use crate::generator::k8sresc::K8sResourceSpec;
use crate::generator::{gen::gen_resource, load_constrained_spec};
use crate::mutator::mutate_resource;
use crate::tui::{tui_loop, tui_restore};

use std::collections::HashMap;
use std::hash::Hasher;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::{fs, thread};
use twox_hash::XxHash64;

static SAMPLE_COUNT: AtomicU64 = AtomicU64::new(0);

struct CorpusEntry<'a> {
    pub data: serde_json::Value,
    pub constraint: &'a K8sResourceSpec,
}

pub struct FuzzingStats {
    pub exit: AtomicBool,
    pub generated: AtomicUsize,
    pub mutated: AtomicUsize,
    pub errors: AtomicUsize,
    pub accepted: AtomicUsize,
    pub corpus_size: AtomicUsize,
    pub newcov: AtomicUsize,
    pub starttime: std::time::Instant,
    pub last_newcov: AtomicU64,   // for these three last fields
    pub last_error: AtomicU64,    // we are using u64s instead of stamps
    pub last_accepted: AtomicU64, // so we can avoid Arc<Mutex>> stuff
}
impl Default for FuzzingStats {
    fn default() -> Self {
        FuzzingStats {
            exit: false.into(),
            generated: 0.into(),
            mutated: 0.into(),
            errors: 0.into(),
            accepted: 0.into(),
            corpus_size: 0.into(),
            newcov: 0.into(),
            starttime: std::time::Instant::now(),
            last_newcov: 0.into(),
            last_error: 0.into(),
            last_accepted: 0.into(),
        }
    }
}

pub async fn run(args: &Fuzz) {
    let client = get_client(args.kubeconfig.as_deref()).await;

    let mut constraintmap = HashMap::<String, K8sResourceSpec>::new();
    let mut corpus = HashMap::<u64, CorpusEntry>::new();

    for file in args.constraints[0].split(",") {
        let cspec = load_constrained_spec(file, &args.schemadir);
        constraintmap.insert(cspec.gvk.clone().unwrap(), cspec);
    }

    // prepare fuzzing directory
    for subdir in ["accepted", "error"] {
        let dir = format!("{}/{}", args.fuzzdir, subdir);
        if !fs::metadata(&dir).is_ok() {
            fs::create_dir(&dir).expect("failed to create fuzzing directory");
        }
    }

    let stats = FuzzingStats::default();
    let fuzzer_stats = Arc::new(stats);
    let tui_stats = fuzzer_stats.clone();

    println!("~ ready to fuzz. Press enter to start ~");
    std::io::stdin()
        .read_line(&mut String::new())
        .expect("failed to read line");

    let tui_thread = thread::spawn(move || {
        match tui_loop(tui_stats) {
            Ok(()) => {}
            Err(e) => {
                tui_restore();
                error_exit!("io error during TUI loop : {}", e);
            }
        }
        tui_restore();
    });

    let mut iter: usize = 0;
    let mut err: FuzzError = FuzzError {
        msg: "".to_string(),
    };

    loop {
        match do_fuzz_iteration(
            &client,
            &mut corpus,
            &constraintmap,
            args,
            fuzzer_stats.clone(),
        )
        .await
        {
            Err(e) => {
                err = e;
                break;
            }
            Ok(()) => {}
        }

        if args.iterations != 0 && iter >= args.iterations
            || fuzzer_stats.exit.load(Ordering::Relaxed)
        {
            break;
        }

        iter += 1;
    }

    fuzzer_stats.exit.store(true, Ordering::Relaxed);
    tui_thread.join().unwrap();
    if err.msg != "" {
        info!("fuzzing stopped due to: {}", err.msg);
    }
    info!("done fuzzing.");
}

fn count_files(directory: &str) -> usize {
    let mut count = 0;
    for entry in fs::read_dir(directory).expect("read_dir call failed") {
        let entry = entry.expect("failed to get entry");
        let path = entry.path();
        if path.is_file() {
            count += 1;
        }
    }
    count
}

enum FuzzResult {
    Accepted,
    Error,
}

fn save_sample(
    sample: &serde_json::Value,
    args: &Fuzz,
    result: FuzzResult,
) -> Result<(), FuzzError> {
    // first get number of saved samples in directory

    let dirpath = format!(
        "{}/{}",
        args.fuzzdir,
        match result {
            FuzzResult::Accepted => "accepted",
            FuzzResult::Error => "error",
        },
    );

    if count_files(&dirpath)
        > match result {
            FuzzResult::Accepted => args.max_accepted,
            FuzzResult::Error => args.max_error,
        }
    {
        return Err(FuzzError {
            msg: format!("max number of samples on disk reached for {}", dirpath),
        });
    }

    // write sample to disk
    let fullpath = format!(
        "{}/{}.json",
        dirpath,
        SAMPLE_COUNT.fetch_add(1, Ordering::SeqCst)
    );

    let f = std::fs::File::create(fullpath).map_err(|e| FuzzError {
        msg: format!("failed to create file: {}", e),
    })?;
    serde_json::to_writer_pretty(f, sample).map_err(|e| FuzzError {
        msg: format!("failed to write sample to file: {}", e),
    })?;

    Ok(())
}

struct FuzzError {
    pub msg: String,
}

async fn submit_and_get_cov(
    client: kube::Client,
    sample: &serde_json::Value,
    args: &Fuzz,
    stats: Arc<FuzzingStats>,
) -> Result<u64, FuzzError> {
    let mut returncode = 0;
    let errormsg = match deploy_resource(&sample, client, &args.namespace).await {
        Ok(_) => {
            // we also want to save it to disk
            save_sample(sample, args, FuzzResult::Accepted)?;
            stats.accepted.fetch_add(1, Ordering::SeqCst);

            "".to_string()
        }
        Err(kube::Error::Api(ae)) => {
            returncode = ae.code;

            if ae.message.contains("connection refused") {
                return Err(FuzzError {
                    msg: format!("connection to admission controller refused: {}", ae.message),
                });
            }

            match ae.status.as_str() {
                "Failure" => match ae.reason.as_str() {
                    "BadRequest" => {
                        // this hints at an error in the resource itself, often
                        // happening due to go unmarshalling errors. So this is
                        // not an error happening in an admission controller. Not
                        // very interesting
                        return Ok(1); // we might want to change this in the future
                    }

                    "Invalid" => {
                        // this hints at semantic errors in the resource, which
                        // we sadly cannot detect because the specification is
                        // not descriptive enough. Still new coverage but we also
                        // do not save it as an error in an admission controller because
                        // it isn't.
                        return Ok(0); // we might want to change this in the future
                    }

                    "InternalError" => {
                        // this is an error happening in an admission controller,
                        // being unable to process the request. We can save this
                        save_sample(sample, args, FuzzResult::Error)?;
                        stats.errors.fetch_add(1, Ordering::SeqCst);

                        stats
                            .last_error
                            .store(stats.starttime.elapsed().as_secs(), Ordering::SeqCst);
                    }
                    _ => {
                        // the reason can be set by the admission controller
                    }
                },
                _ => {
                    return Err(FuzzError {
                        msg: format!(
                            "API returned error that we cannot handle. Please report. {:#?}",
                            ae
                        ),
                    })
                }
            }

            format!("{}{}", ae.reason, ae.message)
        }

        Err(e) => {
            return Err(FuzzError {
                msg: format!("unexpected error type: {:#?}", e),
            })
        }
    };

    Ok(calculate_coverage(
        format!("{}{}", returncode, errormsg).as_str(),
    ))
}

async fn do_fuzz_iteration<'a, 'b>(
    client: &kube::Client,
    corpus: &'a mut HashMap<u64, CorpusEntry<'b>>,
    constraintmap: &'b HashMap<String, K8sResourceSpec>,
    args: &Fuzz,
    stats: Arc<FuzzingStats>,
) -> Result<(), FuzzError> {
    debug!("doing fuzzing iteration");

    let mut newcov = Vec::<(u64, serde_json::Value, &K8sResourceSpec)>::new();

    // mutate corpus, maybe we get some new coverage
    for (_, entry) in &mut *corpus {
        let mut newresource = entry.data.clone();
        mutate_resource(&mut newresource, entry.constraint);
        stats.mutated.fetch_add(1, Ordering::SeqCst);

        let cov = submit_and_get_cov(client.clone(), &newresource, args, stats.clone()).await?;

        newcov.push((cov, newresource, entry.constraint));
    }

    // lets generate fresh manifests
    for (_, constraint) in constraintmap {
        for _ in 0..args.generations {
            let sample = gen_resource(constraint);
            stats.generated.fetch_add(1, Ordering::SeqCst);

            let cov = submit_and_get_cov(client.clone(), &sample, args, stats.clone()).await?;

            newcov.push((cov, sample, constraint));
        }
    }

    // add new coverage to corpus
    for (cov, val, constraint) in newcov {
        if !corpus.contains_key(&cov) {
            let newentry = CorpusEntry {
                data: val,
                constraint: constraint,
            };
            if corpus.len() >= args.max_corpus_count {
                // for now just remove a random entry
                let key = corpus.keys().next().unwrap().clone();
                corpus.remove(&key);
            }
            corpus.insert(cov, newentry);
            stats.newcov.fetch_add(1, Ordering::SeqCst);
            stats
                .last_newcov
                .store(stats.starttime.elapsed().as_secs(), Ordering::SeqCst);
        }
    }

    stats.corpus_size.store(corpus.len(), Ordering::SeqCst);

    Ok(())
}

fn calculate_coverage(errormsg: &str) -> u64 {
    let mut hasher = XxHash64::with_seed(0); // You can use any seed
    hasher.write(errormsg.as_bytes());
    hasher.finish()
}
