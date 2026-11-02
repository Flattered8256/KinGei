import { useRuntimeConfig } from "nitropack/runtime";
import { withoutBase } from "ufo";
import { defu } from "defu";
import { createRouter, toRouteMatcher } from "radix3";
import { normalizeRouteRulePath } from "#internal/nuxt/route-rules.mjs";
//#region src/runtime/utils/route-rules.ts
let matcher;
function getMatcher() {
	if (!matcher) {
		const routes = {};
		for (const [route, rules] of Object.entries(useRuntimeConfig().nitro?.routeRules || {})) routes[normalizeRouteRulePath(route)] = rules;
		matcher = toRouteMatcher(createRouter({ routes }));
	}
	return matcher;
}
/**
* Resolve the route rules for a path without the base URL. Keys and path are decoded,
* and case-folded unless routing is `sensitive`.
*/
function matchRouteRules(path) {
	return defu({}, ...getMatcher().matchAll(normalizeRouteRulePath(path)).reverse());
}
/** Resolve the route rules for a request, as {@link matchRouteRules} does for its path. */
function resolveRequestRouteRules(event) {
	return matchRouteRules(withoutBase(event.path.split("?")[0], useRuntimeConfig().app.baseURL));
}
/** The route rules for a request, reusing those already resolved for it. */
function getRouteRules(event) {
	const context = event.context._nitro ||= {};
	return context.routeRules ||= resolveRequestRouteRules(event);
}
//#endregion
export { getRouteRules, matchRouteRules, resolveRequestRouteRules };
