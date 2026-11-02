import type { NuxtApp } from '#app';
import type { I18nHeadMetaInfo, I18nHeadOptions } from '#internal-i18n-types';
import type { NuxtI18nContext } from './context.js';
import type { RoutingContext } from './routing/context.js';
/**
 * Common options used internally by composable functions, these
 * are initialized on request at the start of i18n:plugin.
 *
 * Extends the platform-neutral {@link RoutingContext} with Nuxt-specific
 * head/SEO state and the client hydration payload.
 *
 * @internal
 */
export type ComposableContext = RoutingContext & {
    strictSeo: boolean;
    routingOptions: {
        defaultLocale: string;
        /** Use `canonicalQueries` for alternate links */
        strictCanonicals: boolean;
        /** Enable/disable hreflangLinks */
        hreflangLinks: boolean;
        /** Whether locales are resolved from domains */
        domains: boolean;
    };
    head: ReturnType<typeof import('nuxt/app').useHead>;
    _head: ReturnType<typeof import('nuxt/app').useHead> | undefined;
    metaState: Required<I18nHeadMetaInfo>;
    seoSettings: I18nHeadOptions;
    localePathPayload: Record<string, Record<string, string> | false>;
};
export declare function useComposableContext(nuxtApp: NuxtApp): ComposableContext;
export declare function createComposableContext(ctx: NuxtI18nContext, nuxtApp?: NuxtApp): ComposableContext;
declare global {
    interface Window {
        _i18nSlp: Record<string, Record<string, unknown> | false> | undefined;
    }
}
