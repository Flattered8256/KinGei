import { serverDiagnostics } from "./diagnostics.mjs";
import { NUXT_ERROR_SIGNATURE } from "./nuxt/src/app/error.mjs";
import { toPortableEvent } from "./utils/event.mjs";
import { baseURL } from "./utils/paths.mjs";
import { getRouteRules as getRouteRules$1, matchRouteRules } from "./utils/route-rules.mjs";
import { createError as createError$1, deleteCookie as deleteCookie$1, getCookie as getCookie$1, getQuery as getQuery$1, getRequestHeader as getRequestHeader$1, getRequestHeaders as getRequestHeaders$1, getRequestURL as getRequestURL$1, parseCookies as parseCookies$1, readBody as readBody$1, setCookie as setCookie$1, setResponseHeader, setResponseStatus } from "h3";
import { useNitroApp, useRuntimeConfig as useRuntimeConfig$1 } from "nitropack/runtime";
import { decodePath, joinURL } from "ufo";
import { clearSession as clearSession$1, deriveSecret, getRequestHost as getRequestHost$2, getRequestProtocol as getRequestProtocol$2, getRequestURL as getRequestURL$2, getRouterParams as getRouterParams$1, getSession as getSession$1, getValidatedQuery as getValidatedQuery$1, handleCors as handleCors$1, readValidatedBody as readValidatedBody$1, resolveServerFetchInit, updateSession as updateSession$1, useAppConfig, useSession as useSession$1 } from "nuxt/internal/server-default";
//#region src/runtime/server.ts
/**
* The `nuxt/server` implementations for a nitropack v2 build, registered as
* `serverBuild.runtime.server`.
*
* Every name `nuxt/server` exports is exported here. Most are h3 v1's own helpers, which
* take the event rather than reading a web-standard request and response off it, plus the
* few Nuxt adds.
*
* The types come from `nuxt/server` whichever module backs it, so a name missing here is
* a runtime error rather than a type error; `test/server.test.ts` guards that.
*/
const clearSession = /* #__PURE__ */ requirePortableEvent("clearSession", clearSession$1);
const getSession = /* #__PURE__ */ requirePortableEvent("getSession", getSession$1);
const getValidatedQuery = /* #__PURE__ */ requirePortableEvent("getValidatedQuery", getValidatedQuery$1);
const handleCors = /* #__PURE__ */ requirePortableEvent("handleCors", handleCors$1);
const readValidatedBody = /* #__PURE__ */ requirePortableEvent("readValidatedBody", readValidatedBody$1);
const updateSession = /* #__PURE__ */ requirePortableEvent("updateSession", updateSession$1);
const useSession = /* #__PURE__ */ requirePortableEvent("useSession", useSession$1);
function requirePortableEvent(helper, fn) {
	return function(event, ...args) {
		if (event?.["~portable"] !== true && event && "node" in event) {
			const diagnostic = serverDiagnostics.NUXT_E8012({ helper });
			throw createError({
				status: 500,
				statusText: "Server Error",
				message: `[${diagnostic.code}] ${diagnostic.message} ${diagnostic.fix}`
			});
		}
		return fn.call(this, event, ...args);
	};
}
/**
* @see {@link import('nuxt/server').defineEventHandler}
*
* The handler is given the event in the portable shape, so that it can read
* `event.req`/`event.url`/`event.res` as the surface promises.
*/
function defineEventHandler(handler) {
	return (event) => handler(toPortableEvent(event));
}
/**
* @see {@link import('nuxt/server').createError}
*
* h3 v1 recognises an error of its own by the marker its constructor carries, so the error
* has to be one it made; `status`/`statusText` are the names it knows as
* `statusCode`/`statusMessage`.
*/
function createError(input) {
	if (typeof input === "string") return withSignature(createError$1(input));
	const { status, statusText, ...rest } = input;
	return withSignature(createError$1({
		...rest,
		...status === void 0 ? {} : { statusCode: status },
		...statusText === void 0 ? {} : {
			statusMessage: statusText,
			message: input.message ?? statusText
		}
	}));
}
/** Mark an h3 error as Nuxt's, so `isNuxtError()` recognises it either side of the wire. */
function withSignature(error) {
	if (!("__nuxt_error" in error)) Object.defineProperty(error, NUXT_ERROR_SIGNATURE, {
		value: true,
		configurable: false,
		writable: false
	});
	return withPortableStatus(error);
}
/**
* Read the status of an h3 v1 error under the names the portable surface promises.
*
* h3 v1 carries it as `statusCode`/`statusMessage`, and `NuxtErrorLike` declares
* `status`/`statusText`, so without this a handler that branches on `error.status` reads
* `undefined` here and a number on a server runtime built against h3 v2.
*/
function withPortableStatus(error) {
	const target = error;
	if (target.status === void 0 && target.statusCode !== void 0) Object.defineProperty(target, "status", {
		get: () => target.statusCode,
		configurable: true
	});
	if (target.statusText === void 0 && target.statusMessage !== void 0) Object.defineProperty(target, "statusText", {
		get: () => target.statusMessage,
		configurable: true
	});
	return target;
}
/**
* @see {@link import('nuxt/server').isNuxtError}
*
* h3 v1 carries the status as `statusCode`, and identifies its own errors by a marker on
* the constructor rather than by name.
*/
function isNuxtError(error) {
	if (!(error instanceof Error)) return false;
	const candidate = error;
	if (typeof candidate.status !== "number" && typeof candidate.statusCode !== "number") return false;
	if (!("__nuxt_error" in error) && candidate.constructor?.__h3_error__ !== true) return false;
	withPortableStatus(error);
	return true;
}
/** @see {@link import('nuxt/server').getRequestURL} */
function getRequestURL(event, options = {}) {
	return getRequestURL$2(toForwardedEvent(event, options), options);
}
/** @see {@link import('nuxt/server').getRequestHost} */
function getRequestHost(event, options = {}) {
	return getRequestHost$2({ req: { headers: event.headers } }, options);
}
/** @see {@link import('nuxt/server').getRequestProtocol} */
function getRequestProtocol(event, options = {}) {
	return getRequestProtocol$2(toForwardedEvent(event, options), options);
}
/** The URL h3 resolves for the request, with the headers to read forwarded values from. */
function toForwardedEvent(event, options) {
	return {
		req: { headers: event.headers },
		url: getRequestURL$1(event, { xForwardedProto: options.xForwardedProto })
	};
}
/** @see {@link import('nuxt/server').getRequestHeader} */
function getRequestHeader(event, name) {
	return getRequestHeader$1(event, name) || void 0;
}
/** @see {@link import('nuxt/server').getRequestHeaders} */
function getRequestHeaders(event) {
	const headers = {};
	for (const [name, value] of Object.entries(getRequestHeaders$1(event))) if (value !== void 0) headers[name] = value;
	return headers;
}
/**
* @see {@link import('nuxt/server').getRequestIP}
*
* The connection address comes from the node socket.
*/
function getRequestIP(event, options = {}) {
	if (options.xForwardedFor) {
		const forwarded = getRequestHeader$1(event, "x-forwarded-for")?.split(",")[0].trim();
		if (forwarded) return forwarded;
	}
	return event.context.clientAddress || event.node.req.socket?.remoteAddress || void 0;
}
/**
* @see {@link import('nuxt/server').getRouterParams}
*
* nitropack v2 routes the decoded path, so h3 v1 matches decoded params. They are read back
* from the same segments of the request URL, so they are percent-encoded as they appear in it.
*/
function getRouterParams(event, options) {
	return getRouterParams$1({ context: { params: getEncodedParams(event) } }, options);
}
/** @see {@link import('nuxt/server').getRouterParam} */
function getRouterParam(event, name, options) {
	return getRouterParams(event, options)[name];
}
function getEncodedParams(event) {
	const params = event.context.params || {};
	const route = event.context.matchedRoute?.path;
	const url = event.node.req.originalUrl || event.node.req.url || "";
	const query = url.includes("?") ? url.slice(url.indexOf("?")) : "";
	if (!route || !event.path.endsWith(query)) return params;
	const raw = url.slice(0, url.length - query.length).split("/");
	const decoded = event.path.slice(0, event.path.length - query.length);
	const offset = raw.length - decoded.split("/").length;
	if (offset < 0) return params;
	const end = decoded.indexOf("?");
	const matched = (end === -1 ? decoded : decoded.slice(0, end)).split("/");
	const last = matched.length - 1;
	if (last > 1 && matched[last] === "") matched.pop();
	const segments = raw.slice(offset, offset + matched.length);
	if (end !== -1 && last < segments.length) segments[last] = segments[last].replace(ENCODED_QUERY_RE, "");
	const encoded = { ...params };
	let unnamed = 0;
	for (const [index, segment] of route.split("/").entries()) {
		const catchAll = segment.startsWith("**");
		const name = catchAll ? segment.slice(3) || "_" : segment === "*" ? `_${unnamed++}` : segment.startsWith(":") ? segment.slice(1) : void 0;
		if (name === void 0) continue;
		const value = catchAll ? segments.slice(index).join("/") : segments[index];
		if (value === void 0 || decodeH3Path(value) !== params[name]) return params;
		encoded[name] = value;
		if (catchAll) break;
	}
	return encoded;
}
const ENCODED_QUERY_RE = /%3f[\s\S]*$/i;
const ENCODED_PERCENT_RE = /%25/g;
/** Decodes a path the way h3 v1 does before routing, keeping `%25` and `%2F` encoded. */
function decodeH3Path(path) {
	return decodePath(path.replace(ENCODED_PERCENT_RE, "%2525"));
}
/** @see {@link import('nuxt/server').readBody} */
function readBody(event) {
	return readBody$1(event);
}
/** @see {@link import('nuxt/server').parseCookies} */
function parseCookies(event) {
	return parseCookies$1(event);
}
/** @see {@link import('nuxt/server').getCookie} */
function getCookie(event, name) {
	return getCookie$1(event, name);
}
/** @see {@link import('nuxt/server').setCookie} */
function setCookie(event, name, value, options) {
	setCookie$1(event, name, value, options);
}
/** @see {@link import('nuxt/server').deleteCookie} */
function deleteCookie(event, name, options) {
	deleteCookie$1(event, name, options);
}
/** @see {@link import('nuxt/server').getQuery} */
function getQuery(event) {
	return getQuery$1(event);
}
/**
* @see {@link import('nuxt/server').sendRedirect}
*
* h3 v1's own `sendRedirect` writes the response and resolves to nothing, where the
* portable surface sets the response and returns the body to respond with. The body is
* built here rather than taken from h3, so a handler that returns it behaves the same on
* every server runtime.
*/
function sendRedirect(event, location, status = 302) {
	setResponseStatus(event, status);
	setResponseHeader(event, "location", location);
	setResponseHeader(event, "content-type", "text/html");
	return `<!DOCTYPE html><html><head><meta http-equiv="refresh" content="0; url=${location.replace(REDIRECT_UNSAFE_RE, (char) => REDIRECT_ESCAPES[char])}"></head></html>`;
}
const REDIRECT_ESCAPES = {
	"\"": "%22",
	"'": "%27",
	"<": "%3C",
	">": "%3E",
	"&": "%26"
};
const REDIRECT_UNSAFE_RE = /["'<>&]/g;
/** @see {@link import('nuxt/server').getRouteRules} */
function getRouteRules(event) {
	return getRouteRules$1(event);
}
/** @see {@link import('nuxt/server').serverFetch} */
function serverFetch(event, path, init) {
	return useNitroApp().localFetch(joinURL(baseURL(), path), resolveServerFetchInit({ req: { headers: event.headers } }, init));
}
/** @see {@link import('nuxt/server').useServerHooks} */
function useServerHooks() {
	return useNitroApp().hooks;
}
/** @see {@link import('nuxt/server').useRuntimeConfig} */
function useRuntimeConfig(event) {
	return useRuntimeConfig$1(event);
}
//#endregion
export { clearSession, createError, defineEventHandler, deleteCookie, deriveSecret, getCookie, getQuery, getRequestHeader, getRequestHeaders, getRequestHost, getRequestIP, getRequestProtocol, getRequestURL, getRouteRules, getRouterParam, getRouterParams, getSession, getValidatedQuery, handleCors, isNuxtError, matchRouteRules, parseCookies, readBody, readValidatedBody, sendRedirect, serverFetch, setCookie, setResponseStatus, updateSession, useAppConfig, useRuntimeConfig, useServerHooks, useSession };
