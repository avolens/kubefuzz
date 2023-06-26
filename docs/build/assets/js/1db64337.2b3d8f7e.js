"use strict";(self.webpackChunkkubefuzz=self.webpackChunkkubefuzz||[]).push([[372],{4137:(e,t,n)=>{n.d(t,{Zo:()=>l,kt:()=>f});var r=n(7294);function o(e,t,n){return t in e?Object.defineProperty(e,t,{value:n,enumerable:!0,configurable:!0,writable:!0}):e[t]=n,e}function i(e,t){var n=Object.keys(e);if(Object.getOwnPropertySymbols){var r=Object.getOwnPropertySymbols(e);t&&(r=r.filter((function(t){return Object.getOwnPropertyDescriptor(e,t).enumerable}))),n.push.apply(n,r)}return n}function a(e){for(var t=1;t<arguments.length;t++){var n=null!=arguments[t]?arguments[t]:{};t%2?i(Object(n),!0).forEach((function(t){o(e,t,n[t])})):Object.getOwnPropertyDescriptors?Object.defineProperties(e,Object.getOwnPropertyDescriptors(n)):i(Object(n)).forEach((function(t){Object.defineProperty(e,t,Object.getOwnPropertyDescriptor(n,t))}))}return e}function s(e,t){if(null==e)return{};var n,r,o=function(e,t){if(null==e)return{};var n,r,o={},i=Object.keys(e);for(r=0;r<i.length;r++)n=i[r],t.indexOf(n)>=0||(o[n]=e[n]);return o}(e,t);if(Object.getOwnPropertySymbols){var i=Object.getOwnPropertySymbols(e);for(r=0;r<i.length;r++)n=i[r],t.indexOf(n)>=0||Object.prototype.propertyIsEnumerable.call(e,n)&&(o[n]=e[n])}return o}var c=r.createContext({}),u=function(e){var t=r.useContext(c),n=t;return e&&(n="function"==typeof e?e(t):a(a({},t),e)),n},l=function(e){var t=u(e.components);return r.createElement(c.Provider,{value:t},e.children)},p="mdxType",d={inlineCode:"code",wrapper:function(e){var t=e.children;return r.createElement(r.Fragment,{},t)}},m=r.forwardRef((function(e,t){var n=e.components,o=e.mdxType,i=e.originalType,c=e.parentName,l=s(e,["components","mdxType","originalType","parentName"]),p=u(n),m=o,f=p["".concat(c,".").concat(m)]||p[m]||d[m]||i;return n?r.createElement(f,a(a({ref:t},l),{},{components:n})):r.createElement(f,a({ref:t},l))}));function f(e,t){var n=arguments,o=t&&t.mdxType;if("string"==typeof e||o){var i=n.length,a=new Array(i);a[0]=m;var s={};for(var c in t)hasOwnProperty.call(t,c)&&(s[c]=t[c]);s.originalType=e,s[p]="string"==typeof e?e:o,a[1]=s;for(var u=2;u<i;u++)a[u]=n[u];return r.createElement.apply(null,a)}return r.createElement.apply(null,n)}m.displayName="MDXCreateElement"},1903:(e,t,n)=>{n.r(t),n.d(t,{assets:()=>c,contentTitle:()=>a,default:()=>d,frontMatter:()=>i,metadata:()=>s,toc:()=>u});var r=n(7462),o=(n(7294),n(4137));const i={sidebar_position:2},a="Overview",s={unversionedId:"overview",id:"overview",title:"Overview",description:"Kubefuzz is a generative and mutative fuzzer for Kubernetes admission controller chains. It can be used to uncover unexpected behavior in complex admission controller setups. It works by generating and mutating Kubernetes resources according to the schema supplied by the cluster openapi scheme, and a user written constrain configuration that further limits what fields are generated and how.",source:"@site/docs/overview.md",sourceDirName:".",slug:"/overview",permalink:"/docs/overview",draft:!1,tags:[],version:"current",sidebarPosition:2,frontMatter:{sidebar_position:2},sidebar:"tutorialSidebar",previous:{title:"Building and Installing",permalink:"/docs/intro"},next:{title:"Get-schmas mode",permalink:"/docs/get-schemas"}},c={},u=[{value:"Modes of operation",id:"modes-of-operation",level:2}],l={toc:u},p="wrapper";function d(e){let{components:t,...n}=e;return(0,o.kt)(p,(0,r.Z)({},l,n,{components:t,mdxType:"MDXLayout"}),(0,o.kt)("h1",{id:"overview"},"Overview"),(0,o.kt)("p",null,"Kubefuzz is a generative and mutative fuzzer for Kubernetes admission controller chains. It can be used to uncover unexpected behavior in complex admission controller setups. It works by generating and mutating Kubernetes resources according to the schema supplied by the cluster openapi scheme, and a user written constrain configuration that further limits what fields are generated and how."),(0,o.kt)("h2",{id:"modes-of-operation"},"Modes of operation"),(0,o.kt)("pre",null,(0,o.kt)("code",{parentName:"pre",className:"language-terminal"},"user@lnx ~> kubefuzz --help\nUsage: kubefuzz [OPTIONS] <COMMAND>\n\nCommands:\n  generate     generate manifests with constraints\n  mutate       mutate existing manifests with constraints\n  fuzz         fuzz admission controller chain with constraints\n  get-schemas  get json schemas from k8s api\n  help         Print this message or the help of the given subcommand(s)\n\nOptions:\n  -s, --seed <SEED>  seed to use\n  -h, --help         Print help\n  -V, --version      Print version\n")),(0,o.kt)("p",null,"Kubefuzz can be run in 4 modes. To be able to use fuzzing, understanding the ",(0,o.kt)("inlineCode",{parentName:"p"},"fuzz")," and the ",(0,o.kt)("inlineCode",{parentName:"p"},"get-schemas")," mode and ",(0,o.kt)("inlineCode",{parentName:"p"},"constraints")," is required."))}d.isMDXComponent=!0}}]);