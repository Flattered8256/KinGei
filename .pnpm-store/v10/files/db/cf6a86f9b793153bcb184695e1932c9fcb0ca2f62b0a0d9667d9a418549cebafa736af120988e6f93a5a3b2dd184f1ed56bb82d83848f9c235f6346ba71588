import { type Locale } from '#i18n';
import type { AllowedComponentProps, AnchorHTMLAttributes, DefineSetupFnComponent, SlotsType, UnwrapRef, VNode, VNodeProps } from 'vue';
import type { NuxtApp, NuxtLinkProps } from 'nuxt/app';
import type { RouteLocation, UseLinkReturn } from 'vue-router';
type NuxtLinkLocaleProps<CustomProp extends boolean = false> = Omit<NuxtLinkProps<CustomProp>, 'to'> & {
    to?: import('vue-router').RouteLocationNamedI18n;
    locale?: Locale;
};
type NuxtLinkLocaleSlotProps<CustomProp extends boolean = false> = CustomProp extends true ? {
    href: string;
    navigate: (e?: MouseEvent) => Promise<void>;
    prefetch: (nuxtApp?: NuxtApp) => Promise<void>;
    route: (RouteLocation & {
        href: string;
    }) | undefined;
    rel: string | null;
    target: '_blank' | '_parent' | '_self' | '_top' | (string & {}) | null;
    isExternal: boolean;
    isActive: false;
    isExactActive: false;
} : UnwrapRef<UseLinkReturn>;
type NuxtLinkLocaleSlots<CustomProp extends boolean = false> = {
    default?: (props: NuxtLinkLocaleSlotProps<CustomProp>) => VNode[];
};
declare const _default: new <CustomProp extends boolean = false>(props: NuxtLinkLocaleProps<CustomProp> & VNodeProps & AllowedComponentProps & Omit<AnchorHTMLAttributes, keyof NuxtLinkLocaleProps<CustomProp>>) => InstanceType<DefineSetupFnComponent<NuxtLinkLocaleProps<CustomProp> & VNodeProps & AllowedComponentProps & Omit<AnchorHTMLAttributes, keyof NuxtLinkLocaleProps<CustomProp>>, [], SlotsType<NuxtLinkLocaleSlots<CustomProp>>>>;
export default _default;
