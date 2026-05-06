/**
 * KeePassEx Mobile — Metro bundler configuration
 * Supports monorepo workspace packages
 */
const { getDefaultConfig, mergeConfig } = require('@react-native/metro-config');
const path = require('path');

const projectRoot = __dirname;
const workspaceRoot = path.resolve(projectRoot, '../..');

const config = {
  watchFolders: [workspaceRoot],

  resolver: {
    // Allow Metro to resolve workspace packages
    nodeModulesPaths: [
      path.resolve(projectRoot, 'node_modules'),
      path.resolve(workspaceRoot, 'node_modules'),
    ],
    // Resolve workspace package aliases
    extraNodeModules: {
      '@keepassex/ui': path.resolve(workspaceRoot, 'packages/ui/src'),
      '@keepassex/i18n': path.resolve(workspaceRoot, 'packages/i18n/src'),
      '@keepassex/types': path.resolve(workspaceRoot, 'shared/types/src'),
      '@keepassex/constants': path.resolve(workspaceRoot, 'shared/constants/src'),
      '@keepassex/utils': path.resolve(workspaceRoot, 'shared/utils/src'),
    },
  },

  transformer: {
    getTransformOptions: async () => ({
      transform: {
        experimentalImportSupport: false,
        inlineRequires: true,
      },
    }),
  },
};

module.exports = mergeConfig(getDefaultConfig(projectRoot), config);
