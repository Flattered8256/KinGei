import { resolveRequestRouteRules } from "../utils/route-rules.mjs";
//#region src/runtime/plugins/route-rules.ts
var route_rules_default = (nitroApp) => {
	nitroApp.hooks.hook("request", (event) => {
		event.context._nitro ||= {};
		event.context._nitro.routeRules = resolveRequestRouteRules(event);
	});
};
//#endregion
export { route_rules_default as default };
