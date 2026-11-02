import { defineNuxtModule } from '@nuxt/kit';
import { isGlobalInstall } from '../dirs.mjs';

const E=globalThis.process?.env||Object.create(null),I=globalThis.process||{env:E},O=void 0!==I&&I.env&&I.env.NODE_ENV||void 0;const A=[["APPVEYOR"],["AWS_AMPLIFY","AWS_APP_ID",{ci:true}],["AZURE_PIPELINES","SYSTEM_TEAMFOUNDATIONCOLLECTIONURI"],["AZURE_STATIC","INPUT_AZURE_STATIC_WEB_APPS_API_TOKEN"],["APPCIRCLE","AC_APPCIRCLE"],["BAMBOO","bamboo_planKey"],["BITBUCKET","BITBUCKET_COMMIT"],["BITRISE","BITRISE_IO"],["BUDDY","BUDDY_WORKSPACE_ID"],["BUILDKITE"],["CIRCLE","CIRCLECI"],["CIRRUS","CIRRUS_CI"],["CLOUDFLARE_PAGES","CF_PAGES",{ci:true}],["CLOUDFLARE_WORKERS","WORKERS_CI",{ci:true}],["GOOGLE_CLOUDRUN","K_SERVICE"],["GOOGLE_CLOUDRUN_JOB","CLOUD_RUN_JOB"],["CODEBUILD","CODEBUILD_BUILD_ARN"],["CODEFRESH","CF_BUILD_ID"],["DRONE"],["DRONE","DRONE_BUILD_EVENT"],["DSARI"],["GITHUB_ACTIONS"],["GITLAB","GITLAB_CI"],["GITLAB","CI_MERGE_REQUEST_ID"],["GOCD","GO_PIPELINE_LABEL"],["LAYERCI"],["JENKINS","JENKINS_URL"],["HUDSON","HUDSON_URL"],["MAGNUM"],["NETLIFY"],["NETLIFY","NETLIFY_LOCAL",{ci:false}],["NEVERCODE"],["RENDER"],["SAIL","SAILCI"],["SEMAPHORE"],["SCREWDRIVER"],["SHIPPABLE"],["SOLANO","TDDIUM"],["STRIDER"],["TEAMCITY","TEAMCITY_VERSION"],["TRAVIS"],["VERCEL","NOW_BUILDER"],["VERCEL","VERCEL",{ci:false}],["VERCEL","VERCEL_ENV",{ci:false}],["APPCENTER","APPCENTER_BUILD_ID"],["CODESANDBOX","CODESANDBOX_SSE",{ci:false}],["CODESANDBOX","CODESANDBOX_HOST",{ci:false}],["STACKBLITZ"],["STORMKIT"],["CLEAVR"],["ZEABUR"],["CODESPHERE","CODESPHERE_APP_ID",{ci:true}],["RAILWAY","RAILWAY_PROJECT_ID"],["RAILWAY","RAILWAY_SERVICE_ID"],["DENO-DEPLOY","DENO_DEPLOY"],["DENO-DEPLOY","DENO_DEPLOYMENT_ID"],["FIREBASE_APP_HOSTING","FIREBASE_APP_HOSTING",{ci:true}],["EDGEONE_PAGES","EO_PAGES_CI",{ci:true}]];function C(){for(const I of A){const O=I[1]||I[0];if(E[O])return {name:I[0].toLowerCase(),...I[2]}}return "/bin/jsh"===E.SHELL&&I.versions?.webcontainer?{name:"stackblitz",ci:false}:{name:"",ci:false}}const D=/* #__PURE__ */C(),T=/* #__PURE__ */(()=>D.name)();I.platform||"";!!I.stdout?.isTTY;!!E.DEBUG;"test"===O||!!E.TEST;"production"===O||"production"===E.MODE;"dev"===O||"development"===O||"development"===E.MODE;const d=(I.versions?.node||"").replace(/^v/,"")||null;Number(d?.split(".")[0])||null;const f=!!I?.versions?.node;!!I?.versions?.nub;const g="Bun"in globalThis,Y="Deno"in globalThis,p="fastly"in globalThis,b="Netlify"in globalThis,m="EdgeRuntime"in globalThis,v="Cloudflare-Workers"===globalThis.navigator?.userAgent,V=[[b,"netlify"],[m,"edge-light"],[v,"workerd"],[p,"fastly"],[Y,"deno"],[g,"bun"],[f,"node"]];const H=function(){const E=V.find(E=>E[0]);if(E)return {name:E[1]}}();H?.name||"";

const WS_EVENT_NAME = "nuxt:devtools:rpc";
const isSandboxed = T === "stackblitz" || T === "codesandbox";
const defaultOptions = {
  enabled: void 0,
  // determine multiple conditions
  componentInspector: true,
  viteInspect: true,
  vscode: {
    enabled: true,
    startOnBoot: false,
    port: 3080,
    reuseExistingServer: true
  },
  disableAuthorization: isSandboxed
};
const defaultTabOptions = {
  behavior: {
    telemetry: null,
    openInEditor: void 0
  },
  ui: {
    componentsView: "list",
    componentsGraphShowNodeModules: false,
    componentsGraphShowGlobalComponents: true,
    componentsGraphShowPages: false,
    componentsGraphShowLayouts: false,
    componentsGraphShowWorkspace: true,
    interactionCloseOnOutsideClick: false,
    showExperimentalFeatures: false,
    showHelpButtons: true,
    showPanel: true,
    scale: 1,
    minimizePanelInactive: 5e3,
    hiddenTabs: [],
    pinnedTabs: [],
    hiddenTabCategories: [],
    sidebarExpanded: false,
    sidebarScrollable: false
  },
  serverRoutes: {
    selectedRoute: null,
    view: "tree",
    inputDefaults: {
      query: [],
      body: [],
      headers: []
    },
    sendFrom: "app"
  },
  serverTasks: {
    enabled: false,
    selectedTask: null,
    view: "list",
    inputDefaults: {
      query: [],
      body: [],
      headers: [{ active: true, key: "Content-Type", value: "application/json", type: "string" }]
    }
  },
  assets: {
    view: "grid"
  }
};
const defaultAllowedExtensions = [
  "png",
  "jpg",
  "jpeg",
  "gif",
  "svg",
  "webp",
  "ico",
  "mp4",
  "ogg",
  "mp3",
  "wav",
  "mov",
  "mkv",
  "mpg",
  "txt",
  "ttf",
  "woff",
  "woff2",
  "eot",
  "json",
  "js",
  "jsx",
  "ts",
  "tsx",
  "md",
  "mdx",
  "vue",
  "webm"
];

const module$1 = defineNuxtModule({
  meta: {
    name: "@nuxt/devtools",
    configKey: "devtools"
  },
  defaults: defaultOptions,
  setup(options, nuxt) {
    if (process.env.VITEST || process.env.TEST)
      return;
    if (typeof options === "boolean")
      options = { enabled: options };
    if (options.enabled === false)
      return;
    if (isGlobalInstall()) {
      const globalOptions = nuxt.options.devtoolsGlobal || {};
      if (options.enabled !== true && !globalOptions.projects?.includes(nuxt.options.rootDir))
        return;
    }
    return import('../chunks/module-main.mjs').then(function (n) { return n.m; }).then(({ enableModule }) => enableModule(options, nuxt));
  }
});

export { WS_EVENT_NAME as W, defaultTabOptions as a, defaultAllowedExtensions as d, module$1 as m };
