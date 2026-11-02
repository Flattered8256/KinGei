import type { NuxtApp } from '#app';
import type { DetectBrowserLanguageOptions, I18nPublicRuntimeConfig, RootRedirectOptions } from '#internal-i18n-types';
import type { H3Event } from 'h3';
export declare function useRuntimeI18n(nuxtApp?: NuxtApp, event?: H3Event): I18nPublicRuntimeConfig;
export declare function useI18nDetection(nuxtApp: NuxtApp | undefined): DetectBrowserLanguageOptions & {
    enabled: boolean;
    cookieKey: string;
};
export declare function resolveRootRedirect(config: string | RootRedirectOptions | undefined): {
    path: string;
    code: number;
} | undefined;
export declare function toArray<T>(value: T | T[]): T[];
/**
 * Escapes a value interpolated into raw HTML, same character set as vue's `escapeHtml`
 * so a rewritten attribute is identical to what vue rendered for the same value.
 *
 * Vue exposes this through `ssrRenderAttr` from `vue/server-renderer`, but importing that
 * would pull the server renderer into the client bundle.
 */
export declare function escapeHtmlAttr(value: string): string;
