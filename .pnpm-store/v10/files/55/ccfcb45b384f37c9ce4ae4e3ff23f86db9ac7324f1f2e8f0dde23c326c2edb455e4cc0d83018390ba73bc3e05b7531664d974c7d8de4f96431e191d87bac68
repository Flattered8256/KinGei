import { createPathIndexLanguageParser } from "@intlify/utils";
import { withTrailingSlash, withoutTrailingSlash } from "ufo";
const separator = __ROUTE_NAME_SEPARATOR__ || "___";
const defaultSuffix = __ROUTE_NAME_DEFAULT_SUFFIX__ || "default";
export const defaultRouteNameSuffix = separator + defaultSuffix;
export function normalizeRouteName(routeName) {
  if (typeof routeName === "string") {
    return routeName;
  }
  if (routeName != null) {
    return routeName.toString();
  }
  return "";
}
export function getRouteBaseName(route) {
  return normalizeRouteName(typeof route === "object" ? route?.name : route).split(separator)[0];
}
export function getLocalizedRouteName(routeName, locale, isDefault) {
  return !isDefault ? routeName + separator + locale : routeName + separator + locale + defaultRouteNameSuffix;
}
export const createTrailingSlashFormatter = (trailingSlash) => trailingSlash ? withTrailingSlash : withoutTrailingSlash;
export function prefixable(currentLocale, defaultLocale, options) {
  return options.routing && (currentLocale !== defaultLocale || options.strategy === "prefix");
}
const pathLanguageParser = createPathIndexLanguageParser(0);
export const getLocaleFromRoutePath = (path) => pathLanguageParser(path);
export const getLocaleFromRouteName = (name) => name.split(separator).at(1) ?? "";
function normalizeInput(input) {
  return typeof input !== "object" ? String(input) : String(input?.name || input?.path || "");
}
export function getLocaleFromRoute(route) {
  const input = normalizeInput(route);
  if (input[0] === "/") {
    return getLocaleFromRoutePath(input);
  }
  const fromName = getLocaleFromRouteName(input);
  if (fromName) {
    return fromName;
  }
  if (typeof route === "object" && route?.path) {
    return getLocaleFromRoutePath(String(route.path));
  }
  return "";
}
