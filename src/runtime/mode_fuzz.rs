use std::collections::HashMap;

use crate::args::Fuzz;
use crate::error_exit;
use crate::executor::{deploy_resource, get_client};
use crate::generator::gen;
use crate::generator::k8sresc::K8sResourceSpec;
use crate::generator::{gen::gen_resource, load_constrained_spec};
use crate::mutator::mutate_resource;
use std::fs;
use std::hash::Hasher;
use twox_hash::XxHash64;

use std::sync::atomic::{AtomicU64, Ordering};

static SAMPLE_COUNT: AtomicU64 = AtomicU64::new(0);

struct CorpusEntry<'a> {
    pub data: serde_json::Value,
    pub constraint: &'a K8sResourceSpec,
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

    if args.iterations == 0 {
        loop {
            do_fuzz_iteration(&client, &mut corpus, &constraintmap, args).await;
        }
        for _ in 0..args.iterations {
            do_fuzz_iteration(&client, &mut corpus, &constraintmap, args).await;
        }
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

fn save_sample(sample: &serde_json::Value, args: &Fuzz, result: FuzzResult) {
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
        info!("max disk corpus count reached in {}. Stopping.", &dirpath);
        std::process::exit(0);
    }

    // write sample to disk
    let fullpath = format!(
        "{}/{}.json",
        dirpath,
        SAMPLE_COUNT.fetch_add(1, Ordering::SeqCst)
    );
    let f = std::fs::File::create(fullpath).expect("failed to create file for sample");
    serde_json::to_writer_pretty(f, sample).expect("failed to write sample to file");
}

async fn submit_and_get_cov(client: kube::Client, sample: &serde_json::Value, args: &Fuzz) -> u64 {
    let mut returncode = 0;
    let errormsg = match deploy_resource(&sample, client, &args.namespace).await {
        Ok(_) => {
            // we also want to save it to disk
            save_sample(sample, args, FuzzResult::Accepted);

            "".to_string()
        }
        Err(kube::Error::Api(ae)) => {
            returncode = ae.code;

            if ae.message.contains("connection refused") {
                error_exit!("API returned connection refused for admission controller. Stopping. Answer: {:#?}", ae);
            }

            match ae.status.as_str() {
                "Failure" => match ae.reason.as_str() {
                    "BadRequest" => {
                        // this hints at an error in the resource itself, often
                        // happening due to go unmarshalling errors. So this is
                        // not an error happening in an admission controller. We
                        // add this to the corpus but we dont save it as an error
                    }

                    "Invalid" => {
                        // this hints at semantic errors in the resource, which
                        // we sadly cannot detect because the specification is
                        // not descriptive enough. Still new coverage but we also
                        // do not save it as an error in an admission controller because
                        // it isn't.
                    }

                    // todo: we haveto verify that InternalError is always thrown
                    // for admission controller errors
                    "InternalError" => {
                        // this is an error happening in an admission controller,
                        // being unable to process the request. We can save this
                        save_sample(sample, args, FuzzResult::Error);
                    }
                    _ => {
                        // the reason can be set by the admission controller
                    }
                },
                _ => {
                    error_exit!(
                        "API returned unknown status on error that we cant handle {:#?}. Please report this",
                        ae
                    )
                }
            }

            format!("{}{}", ae.reason, ae.message)
        }

        Err(_) => panic!("unexpected error"),
    };

    calculate_coverage(format!("{}{}", returncode, errormsg).as_str())
}

async fn do_fuzz_iteration<'a, 'b>(
    client: &kube::Client,
    corpus: &'a mut HashMap<u64, CorpusEntry<'b>>,
    constraintmap: &'b HashMap<String, K8sResourceSpec>,
    args: &Fuzz,
) {
    debug!("doing fuzzing iteration");

    let mut newcov = Vec::<(u64, serde_json::Value, &K8sResourceSpec)>::new();

    // mutate corpus, maybe we get some new coverage
    for (_, entry) in &mut *corpus {
        let mut newresource = entry.data.clone();
        mutate_resource(&mut newresource, entry.constraint);

        let cov = submit_and_get_cov(client.clone(), &newresource, args).await;

        newcov.push((cov, newresource, entry.constraint));
    }

    // lets generate fresh manifests
    for (gvk, constraint) in constraintmap {
        let sample = gen_resource(constraint);

        let cov = submit_and_get_cov(client.clone(), &sample, args).await;

        newcov.push((cov, sample, constraint));
    }

    // add new coverage to corpus
    for (cov, val, constraint) in newcov {
        if !corpus.contains_key(&cov) {
            let newentry = CorpusEntry {
                data: val,
                constraint: constraint,
            };
            if corpus.len() > args.max_corpus_count {
                // for now just remove a random entry
                let key = corpus.keys().next().unwrap().clone();
                corpus.remove(&key);
            }
            corpus.insert(cov, newentry);
        }
    }
}

fn calculate_coverage(errormsg: &str) -> u64 {
    let mut hasher = XxHash64::with_seed(0); // You can use any seed
    hasher.write(errormsg.as_bytes());
    hasher.finish()
}
