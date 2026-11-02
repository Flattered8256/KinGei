export function createPrerenderablePredicate(config) {
  return (locale) => !config.dynamic.includes(locale) && !config.undeliverable.includes(locale);
}
export function createRuntimeLoaderPredicate(config) {
  return (locale) => !config.ssr || config.undeliverable.includes(locale) || config.dynamic.includes(locale) && (config.prerender || config.ssg);
}
