"use strict";(self.webpackChunkkubefuzz=self.webpackChunkkubefuzz||[]).push([[443],{4137:(e,t,n)=>{n.d(t,{Zo:()=>l,kt:()=>f});var r=n(7294);function o(e,t,n){return t in e?Object.defineProperty(e,t,{value:n,enumerable:!0,configurable:!0,writable:!0}):e[t]=n,e}function a(e,t){var n=Object.keys(e);if(Object.getOwnPropertySymbols){var r=Object.getOwnPropertySymbols(e);t&&(r=r.filter((function(t){return Object.getOwnPropertyDescriptor(e,t).enumerable}))),n.push.apply(n,r)}return n}function i(e){for(var t=1;t<arguments.length;t++){var n=null!=arguments[t]?arguments[t]:{};t%2?a(Object(n),!0).forEach((function(t){o(e,t,n[t])})):Object.getOwnPropertyDescriptors?Object.defineProperties(e,Object.getOwnPropertyDescriptors(n)):a(Object(n)).forEach((function(t){Object.defineProperty(e,t,Object.getOwnPropertyDescriptor(n,t))}))}return e}function s(e,t){if(null==e)return{};var n,r,o=function(e,t){if(null==e)return{};var n,r,o={},a=Object.keys(e);for(r=0;r<a.length;r++)n=a[r],t.indexOf(n)>=0||(o[n]=e[n]);return o}(e,t);if(Object.getOwnPropertySymbols){var a=Object.getOwnPropertySymbols(e);for(r=0;r<a.length;r++)n=a[r],t.indexOf(n)>=0||Object.prototype.propertyIsEnumerable.call(e,n)&&(o[n]=e[n])}return o}var c=r.createContext({}),u=function(e){var t=r.useContext(c),n=t;return e&&(n="function"==typeof e?e(t):i(i({},t),e)),n},l=function(e){var t=u(e.components);return r.createElement(c.Provider,{value:t},e.children)},p="mdxType",m={inlineCode:"code",wrapper:function(e){var t=e.children;return r.createElement(r.Fragment,{},t)}},d=r.forwardRef((function(e,t){var n=e.components,o=e.mdxType,a=e.originalType,c=e.parentName,l=s(e,["components","mdxType","originalType","parentName"]),p=u(n),d=o,f=p["".concat(c,".").concat(d)]||p[d]||m[d]||a;return n?r.createElement(f,i(i({ref:t},l),{},{components:n})):r.createElement(f,i({ref:t},l))}));function f(e,t){var n=arguments,o=t&&t.mdxType;if("string"==typeof e||o){var a=n.length,i=new Array(a);i[0]=d;var s={};for(var c in t)hasOwnProperty.call(t,c)&&(s[c]=t[c]);s.originalType=e,s[p]="string"==typeof e?e:o,i[1]=s;for(var u=2;u<a;u++)i[u]=n[u];return r.createElement.apply(null,i)}return r.createElement.apply(null,n)}d.displayName="MDXCreateElement"},7711:(e,t,n)=>{n.r(t),n.d(t,{assets:()=>c,contentTitle:()=>i,default:()=>m,frontMatter:()=>a,metadata:()=>s,toc:()=>u});var r=n(7462),o=(n(7294),n(4137));const a={sidebar_position:5},i="Generation mode",s={unversionedId:"generate",id:"generate",title:"Generation mode",description:"In generation mode KubeFuzz will generate random resources according to the user supplied constraint:",source:"@site/docs/generate.md",sourceDirName:".",slug:"/generate",permalink:"/docs/generate",draft:!1,tags:[],version:"current",sidebarPosition:5,frontMatter:{sidebar_position:5},sidebar:"tutorialSidebar",previous:{title:"Constraints",permalink:"/docs/constraints"},next:{title:"Mutation mode",permalink:"/docs/mutate"}},c={},u=[],l={toc:u},p="wrapper";function m(e){let{components:t,...n}=e;return(0,o.kt)(p,(0,r.Z)({},l,n,{components:t,mdxType:"MDXLayout"}),(0,o.kt)("h1",{id:"generation-mode"},"Generation mode"),(0,o.kt)("p",null,"In generation mode KubeFuzz will generate random resources according to the user supplied constraint:"),(0,o.kt)("pre",null,(0,o.kt)("code",{parentName:"pre",className:"language-terminal"},"user@lnx ~> kubefuzz generate --help\ngenerate manifests with constraints\n\nUsage: kubefuzz generate [OPTIONS] --constraints <CONSTRAINTS> --schemadir <SCHEMADIR> --out <OUT>\n\nOptions:\n  -c, --constraints <CONSTRAINTS>  comma seperated list of constraint files to apply\n  -s, --schemadir <SCHEMADIR>      directory containing k8s json resource schemas\n  -o, --out <OUT>                  output direcotry of generated schemas\n  -n, --num <NUM>                  number of manifests to generate per resource [default: 10]\n  -h, --help                       Print help\n\nuser@lnx ~> mkdir out\nuser@lnx ~> kubefuzz generate -c /path/to/constraint.yaml,anotherconstraint.yaml -o out -s /path/to/schemas/\n")))}m.isMDXComponent=!0}}]);