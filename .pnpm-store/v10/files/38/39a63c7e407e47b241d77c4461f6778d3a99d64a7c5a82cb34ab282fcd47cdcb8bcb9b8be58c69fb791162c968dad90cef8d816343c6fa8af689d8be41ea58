import { type Ref } from '#imports';
import type { I18nHeadMetaInfo, I18nHeadOptions, SeoAttributesOptions } from '#internal-i18n-types';
import type { ComposableContext } from '../composable-context.js';
import type { I18nRouteMeta } from '../types.js';
/**
 * Alternate links annotate the domains as one cluster, and a cluster has a single fallback for
 * unmatched languages. It cannot be resolved from the current domain - every domain would name a
 * different one - so `x-default` is left out until `defaultLocale` names it.
 */
export declare function missesClusterFallback(ctx: ComposableContext, config: Required<I18nHeadOptions>): boolean;
/**
 * Returns localized head properties for locale-related aspects.
 *
 * @param ctx - Context used internally by composable functions.
 * @param options - An options, see about details {@link I18nHeadOptions}.
 *
 * @returns The localized {@link I18nHeadMetaInfo | head properties}.
 *
 * @public
 */
export declare function localeHead(ctx: ComposableContext, { dir, lang, seo }: I18nHeadOptions): I18nHeadMetaInfo;
export declare function _useLocaleHead(ctx: ComposableContext, options: Required<I18nHeadOptions>): Ref<I18nHeadMetaInfo>;
export declare function _useSetI18nParams(ctx: ComposableContext, seo?: SeoAttributesOptions, router?: import("vue-router")._RouterClassic): (params: I18nRouteMeta) => void;
