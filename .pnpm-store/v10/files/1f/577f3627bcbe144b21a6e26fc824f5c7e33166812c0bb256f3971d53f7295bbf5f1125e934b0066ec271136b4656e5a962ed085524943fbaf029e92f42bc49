import { useSwitchLocalePath } from "#i18n";
import { defineNuxtLink, useNuxtApp } from "#imports";
import { Comment, computed, defineComponent, h, mergeProps } from "vue";
import { hasProtocol } from "ufo";
import { nuxtLinkDefaults } from "#build/nuxt.config.mjs";
import { useComposableContext } from "../composable-context.js";
import { useNuxtI18nContext } from "../context.js";
const NuxtLink = defineNuxtLink({ ...nuxtLinkDefaults, componentName: "NuxtLink" });
const SlpComponent = defineComponent({
  name: "SwitchLocalePathLink",
  props: {
    locale: {
      type: String,
      required: true
    }
  },
  setup(props, { slots, attrs }) {
    const nuxtApp = useNuxtApp();
    const payload = useComposableContext(nuxtApp).localePathPayload;
    const switchLocalePath = useSwitchLocalePath(nuxtApp);
    const resolved = computed(() => {
      if (__I18N_STRICT_SEO__ && nuxtApp.isHydrating && Object.keys(payload ?? {}).length && !payload?.[props.locale]) {
        return "#";
      }
      return switchLocalePath(props.locale) || __I18N_STRICT_SEO__ && "#" || "";
    });
    const disabled = computed(() => __I18N_STRICT_SEO__ && resolved.value === "#" || void 0);
    const external = computed(() => hasProtocol(resolved.value, { strict: true }));
    const onClick = (event) => __I18N_DOMAINS__ && external.value && !disabled.value && !event.defaultPrevented && useNuxtI18nContext(nuxtApp).setCookieLocale(props.locale);
    return () => h(
      NuxtLink,
      mergeProps(
        { rel: external.value ? "noopener" : void 0, referrerpolicy: external.value ? "origin" : void 0 },
        attrs,
        { "to": resolved.value, "data-i18n-disabled": disabled.value, onClick }
      ),
      slots.default
    );
  }
});
export default defineComponent({
  name: "SwitchLocalePathLinkWrapper",
  props: {
    locale: {
      type: String,
      required: true
    }
  },
  inheritAttrs: false,
  setup(props, { slots, attrs }) {
    return () => [
      h(Comment, `${__SWITCH_LOCALE_PATH_LINK_IDENTIFIER__}-[${props.locale}]`),
      h(SlpComponent, { ...attrs, ...props }, slots.default),
      h(Comment, `/${__SWITCH_LOCALE_PATH_LINK_IDENTIFIER__}`)
    ];
  }
});
