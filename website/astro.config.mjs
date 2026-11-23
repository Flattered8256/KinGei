// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

// https://astro.build/config
export default defineConfig({
	site: 'https://flattered8256.github.io',
	base: '/KinGei',
	integrations: [
		starlight({
			title: 'KinGei',
			logo: {
				src: './src/assets/logo.svg',
				alt: 'KinGei',
			},
			social: [{ icon: 'github', label: 'GitHub', href: 'https://github.com/Flattered8256/KinGei' }],
			customCss: ['./src/styles/custom.css'],
			defaultLocale: 'root',
			locales: {
				root: {
					label: '简体中文',
					lang: 'zh-CN',
				},
				en: {
					label: 'English',
				},
			},
			sidebar: [
				{
					label: '开始',
					translations: { en: 'Start' },
					items: [{ slug: 'guides/docker' }],
				},
			],
		}),
	],
});
