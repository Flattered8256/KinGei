import type { HookInfo } from '@nuxt/devtools/types';
interface HookableLike {
    beforeEach: (fn: (event: {
        name: string;
    }) => void) => void;
    afterEach: (fn: (event: {
        name: string;
    }) => void) => void;
}
export declare function setupHooksDebug<T extends HookableLike>(hooks: T): Record<string, HookInfo>;
export {};
