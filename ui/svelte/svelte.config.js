import adapter from '@sveltejs/adapter-static';

export default {
  kit: {
    adapter: adapter({ out: 'dist', fallback: 'index.html' }),
    paths: { base: '' },
  },
};
