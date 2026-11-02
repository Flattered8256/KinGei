import type { Strategies } from '#internal-i18n-types';
import type { RouteResources } from '../../routing.js';
export type PathMatcherConfig = {
    strategy: Strategies;
    /** Whether routes are localized (pages enabled and strategy is not `no_prefix`) */
    routing: boolean;
    trailingSlash: boolean;
};
/**
 * Resolves request paths against the routes the build generated, so path shapes are matched
 * rather than recomputed. Takes the resources and flags as values, the runtime instance below
 * composes it from the generated ones.
 */
export declare function createPathMatcher(resources: RouteResources, config: PathMatcherConfig): {
    isExistingNuxtRoute: (path: string) => import("vue-router").MatcherLocation | undefined;
    matchLocalized: (path: string, locale: string, defaultLocale: string) => string | undefined;
};
declare const isExistingNuxtRoute: (path: string) => import("vue-router").MatcherLocation | undefined, matchLocalized: (path: string, locale: string, defaultLocale: string) => string | undefined;
export { isExistingNuxtRoute, matchLocalized };
