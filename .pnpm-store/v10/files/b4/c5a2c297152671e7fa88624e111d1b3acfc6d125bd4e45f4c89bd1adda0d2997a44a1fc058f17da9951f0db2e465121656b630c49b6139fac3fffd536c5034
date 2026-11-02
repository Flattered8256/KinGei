import type { DefineLocaleMessage, I18nOptions, Locale, LocaleMessages } from 'vue-i18n';
import type { VueI18nConfig } from '#internal-i18n-types';
type MessageLoaderFunction<T = DefineLocaleMessage> = (locale: Locale) => Promise<LocaleMessages<T>>;
type MessageLoaderResult<T, Result = MessageLoaderFunction<T> | LocaleMessages<T>> = {
    default: Result;
} | Result;
type LocaleLoader<T = LocaleMessages<DefineLocaleMessage>> = {
    key: string;
    cache: boolean;
    load: () => Promise<MessageLoaderResult<T>>;
};
/**
 * Clone a message tree. Unlike `@intlify/shared`'s `deepCopy` this copies arrays instead of
 * sharing them; message functions are shared, vue-i18n treats them as opaque values.
 */
export declare function cloneDeep<T>(value: T): T;
/** Whether a message tree holds a function anywhere - JSON delivery drops these silently */
export declare function hasMessageFunction(value: unknown, seen?: WeakSet<object>): boolean;
/**
 * Backstop for the build-time scan (#3880): warns when a locale it classified as serializable turns
 * out to hold functions after all. Callers gate this to dev/prerender, walking every tree is not free.
 */
export declare function warnMissedMessageFunctions(locale: string, messages: unknown): void;
/**
 * Returns a tree where `base` fills the gaps in `over`, sharing every subtree `base` does not reach.
 * Equivalent to merging `over` into a copy of `base`, but the copying is proportional to `base` -
 * where `mergeLocaleMessage` deep copies proportional to `over`, which is the whole point.
 */
export declare function fillMissing<T>(over: T, base: unknown): T;
export declare function loadVueI18nOptions(vueI18nConfigs: VueI18nConfig[]): Promise<I18nOptions>;
/**
 * Get locale messages from the loaders of a single locale and merge these
 */
export declare function getLocaleMessagesMerged(locale: string, loaders?: LocaleLoader[]): Promise<LocaleMessages<DefineLocaleMessage>>;
/**
 * Wraps the `getLocaleMessages` function to use cache
 */
export declare function getLocaleMessagesMergedCached(locale: string, loaders?: LocaleLoader[]): Promise<LocaleMessages<DefineLocaleMessage>>;
export {};
