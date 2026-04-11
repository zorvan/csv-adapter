/**
 * Template configurations for create-csv-app
 * Each template defines files, dependencies, and environment variables
 */

const CHAINS = {
  ethereum: {
    id: 'ethereum',
    name: 'Ethereum',
    symbol: 'ETH',
    chainId: 1,
    testnetChainId: 11155111,
    testnetName: 'Sepolia',
    rpcEnvVar: 'ETHEREUM_RPC_URL',
    defaultTestnetRpc: 'https://rpc.sepolia.org'
  },
  bitcoin: {
    id: 'bitcoin',
    name: 'Bitcoin',
    symbol: 'BTC',
    chainId: 0,
    testnetChainId: 0,
    testnetName: 'Testnet',
    rpcEnvVar: 'BITCOIN_RPC_URL',
    defaultTestnetRpc: 'https://mempool.space/testnet/api'
  },
  sui: {
    id: 'sui',
    name: 'Sui',
    symbol: 'SUI',
    chainId: 0,
    testnetChainId: 0,
    testnetName: 'Testnet',
    rpcEnvVar: 'SUI_RPC_URL',
    defaultTestnetRpc: 'https://fullnode.testnet.sui.io:443'
  },
  aptos: {
    id: 'aptos',
    name: 'Aptos',
    symbol: 'APT',
    chainId: 1,
    testnetChainId: 2,
    testnetName: 'Testnet',
    rpcEnvVar: 'APTOS_RPC_URL',
    defaultTestnetRpc: 'https://fullnode.testnet.aptoslabs.com/v1'
  }
};

const TEMPLATES = {
  'basic-transfer': {
    id: 'basic-transfer',
    name: 'Basic Transfer',
    description: 'A minimal cross-chain transfer application. Perfect for learning the CSV SDK.',
    category: 'starter',
    githubPath: 'zorvan/csv-adapter-templates/basic-transfer',
    framework: 'vanilla',
    language: 'typescript',
    defaultChains: ['ethereum', 'sui'],
    envVars: {
      MNEMONIC: {
        description: 'BIP-39 mnemonic for dev wallet',
        required: true,
        sensitive: true
      },
      ETHEREUM_RPC_URL: {
        description: 'Ethereum RPC endpoint',
        required: false,
        default: 'https://rpc.sepolia.org'
      },
      SUI_RPC_URL: {
        description: 'Sui RPC endpoint',
        required: false,
        default: 'https://fullnode.testnet.sui.io:443'
      },
      CSV_LOG_LEVEL: {
        description: 'Log level (debug, info, warn, error)',
        required: false,
        default: 'info'
      }
    },
    dependencies: {
      '@csv-adapter/core': 'latest',
      '@csv-adapter/ethereum': 'latest',
      '@csv-adapter/sui': 'latest',
      'dotenv': '^16.4.0',
      'ethers': '^6.11.0',
      '@mysten/sui.js': '^0.54.0'
    },
    scripts: {
      'dev': 'tsx watch src/index.ts',
      'build': 'tsc',
      'start': 'node dist/index.js',
      'transfer': 'tsx src/index.ts transfer'
    },
    files: {
      'src/index.ts': true,
      'tsconfig.json': true,
      '.env.example': true,
      'package.json': true
    },
    nextSteps: [
      'Run the development server: npm run dev',
      'Execute a sample transfer: npm run transfer',
      'Edit src/index.ts to customize the transfer logic',
      'Add more chains by installing additional CSV adapters'
    ]
  },

  'nft-marketplace': {
    id: 'nft-marketplace',
    name: 'NFT Marketplace',
    description: 'A full-stack NFT marketplace with cross-chain minting and trading.',
    category: 'fullstack',
    githubPath: 'zorvan/csv-adapter-templates/nft-marketplace',
    framework: 'nextjs',
    language: 'typescript',
    defaultChains: ['ethereum', 'sui', 'aptos'],
    envVars: {
      MNEMONIC: {
        description: 'BIP-39 mnemonic for dev wallet',
        required: true,
        sensitive: true
      },
      ETHEREUM_RPC_URL: {
        description: 'Ethereum RPC endpoint',
        required: false,
        default: 'https://rpc.sepolia.org'
      },
      SUI_RPC_URL: {
        description: 'Sui RPC endpoint',
        required: false,
        default: 'https://fullnode.testnet.sui.io:443'
      },
      APTOS_RPC_URL: {
        description: 'Aptos RPC endpoint',
        required: false,
        default: 'https://fullnode.testnet.aptoslabs.com/v1'
      },
      NEXT_PUBLIC_APP_NAME: {
        description: 'Application display name',
        required: false,
        default: 'CSV NFT Marketplace'
      },
      CSV_LOG_LEVEL: {
        description: 'Log level (debug, info, warn, error)',
        required: false,
        default: 'info'
      }
    },
    dependencies: {
      '@csv-adapter/core': 'latest',
      '@csv-adapter/ethereum': 'latest',
      '@csv-adapter/sui': 'latest',
      '@csv-adapter/aptos': 'latest',
      'next': '^14.2.0',
      'react': '^18.3.0',
      'react-dom': '^18.3.0',
      '@rainbow-me/rainbowkit': '^2.1.0',
      'wagmi': '^2.5.0',
      'viem': '^2.9.0',
      'ethers': '^6.11.0',
      '@mysten/sui.js': '^0.54.0',
      '@aptos-labs/ts-sdk': '^1.21.0',
      'tailwindcss': '^3.4.0',
      'zustand': '^4.5.0',
      'dotenv': '^16.4.0'
    },
    scripts: {
      'dev': 'next dev',
      'build': 'next build',
      'start': 'next start',
      'lint': 'next lint'
    },
    files: {
      'src/components/NFTCard.tsx': true,
      'src/components/Marketplace.tsx': true,
      'src/components/ConnectWallet.tsx': true,
      'src/components/ChainSelector.tsx': true,
      'src/lib/csv-adapter.ts': true,
      'src/lib/nft-store.ts': true,
      'src/pages/index.tsx': true,
      'src/pages/nft/[id].tsx': true,
      'src/pages/mint.tsx': true,
      'src/pages/api/mint.ts': true,
      'src/pages/api/transfer.ts': true,
      'src/styles/globals.css': true,
      'src/types.ts': true,
      'tsconfig.json': true,
      'next.config.js': true,
      'tailwind.config.js': true,
      '.env.example': true,
      'package.json': true
    },
    nextSteps: [
      'Start the dev server: npm run dev',
      'Open http://localhost:3000 in your browser',
      'Connect your wallet using RainbowKit',
      'Mint your first NFT on any chain',
      'Try cross-chain NFT transfer from the UI'
    ]
  },

  'cross-chain-lending': {
    id: 'cross-chain-lending',
    name: 'Cross-Chain Lending',
    description: 'A backend service for cross-chain lending and borrowing with SQLite persistence.',
    category: 'backend',
    githubPath: 'zorvan/csv-adapter-templates/cross-chain-lending',
    framework: 'express',
    language: 'typescript',
    defaultChains: ['ethereum', 'bitcoin', 'sui'],
    envVars: {
      MNEMONIC: {
        description: 'BIP-39 mnemonic for dev wallet',
        required: true,
        sensitive: true
      },
      ETHEREUM_RPC_URL: {
        description: 'Ethereum RPC endpoint',
        required: false,
        default: 'https://rpc.sepolia.org'
      },
      BITCOIN_RPC_URL: {
        description: 'Bitcoin RPC endpoint',
        required: false,
        default: 'https://mempool.space/testnet/api'
      },
      SUI_RPC_URL: {
        description: 'Sui RPC endpoint',
        required: false,
        default: 'https://fullnode.testnet.sui.io:443'
      },
      DATABASE_URL: {
        description: 'SQLite database file path',
        required: false,
        default: './data/lending.db'
      },
      PORT: {
        description: 'HTTP server port',
        required: false,
        default: '3000'
      },
      API_SECRET: {
        description: 'API authentication secret',
        required: true,
        sensitive: true
      },
      CSV_LOG_LEVEL: {
        description: 'Log level (debug, info, warn, error)',
        required: false,
        default: 'info'
      }
    },
    dependencies: {
      '@csv-adapter/core': 'latest',
      '@csv-adapter/ethereum': 'latest',
      '@csv-adapter/bitcoin': 'latest',
      '@csv-adapter/sui': 'latest',
      'express': '^4.18.0',
      'better-sqlite3': '^9.4.0',
      'dotenv': '^16.4.0',
      'zod': '^3.22.0',
      'cors': '^2.8.5',
      'helmet': '^7.1.0',
      'morgan': '^1.10.0',
      'ethers': '^6.11.0',
      '@mysten/sui.js': '^0.54.0'
    },
    scripts: {
      'dev': 'tsx watch src/index.ts',
      'build': 'tsc',
      'start': 'node dist/index.js',
      'migrate': 'tsx src/db/migrate.ts',
      'seed': 'tsx src/db/seed.ts'
    },
    files: {
      'src/index.ts': true,
      'src/app.ts': true,
      'src/routes/loans.ts': true,
      'src/routes/collateral.ts': true,
      'src/routes/health.ts': true,
      'src/db/database.ts': true,
      'src/db/migrate.ts': true,
      'src/db/seed.ts': true,
      'src/middleware/auth.ts': true,
      'src/middleware/error.ts': true,
      'src/types.ts': true,
      'tsconfig.json': true,
      '.env.example': true,
      'package.json': true
    },
    nextSteps: [
      'Run database migrations: npm run migrate',
      'Start the dev server: npm run dev',
      'Create your first loan: POST /api/loans',
      'Deposit collateral using the /api/collateral endpoint',
      'Monitor loan health with the built-in health checks'
    ]
  },

  'event-ticketing': {
    id: 'event-ticketing',
    name: 'Event Ticketing',
    description: 'A mobile-friendly ticketing app with QR codes and cross-chain validation.',
    category: 'frontend',
    githubPath: 'zorvan/csv-adapter-templates/event-ticketing',
    framework: 'react-vite',
    language: 'typescript',
    defaultChains: ['ethereum', 'aptos'],
    envVars: {
      MNEMONIC: {
        description: 'BIP-39 mnemonic for dev wallet',
        required: true,
        sensitive: true
      },
      ETHEREUM_RPC_URL: {
        description: 'Ethereum RPC endpoint',
        required: false,
        default: 'https://rpc.sepolia.org'
      },
      APTOS_RPC_URL: {
        description: 'Aptos RPC endpoint',
        required: false,
        default: 'https://fullnode.testnet.aptoslabs.com/v1'
      },
      VITE_APP_NAME: {
        description: 'Application display name',
        required: false,
        default: 'CSV Event Tickets'
      },
      VITE_ORGANIZER_ADDRESS: {
        description: 'Event organizer wallet address',
        required: false,
        default: ''
      },
      CSV_LOG_LEVEL: {
        description: 'Log level (debug, info, warn, error)',
        required: false,
        default: 'info'
      }
    },
    dependencies: {
      '@csv-adapter/core': 'latest',
      '@csv-adapter/ethereum': 'latest',
      '@csv-adapter/aptos': 'latest',
      'react': '^18.3.0',
      'react-dom': '^18.3.0',
      'react-qr-code': '^2.0.12',
      'qrcode.react': '^3.1.0',
      'html5-qrcode': '^2.3.8',
      'vite': '^5.2.0',
      '@vitejs/plugin-react': '^4.2.0',
      'tailwindcss': '^3.4.0',
      'ethers': '^6.11.0',
      '@aptos-labs/ts-sdk': '^1.21.0',
      'zustand': '^4.5.0',
      'date-fns': '^3.3.0'
    },
    scripts: {
      'dev': 'vite',
      'build': 'vite build',
      'preview': 'vite preview',
      'generate-qr': 'tsx scripts/generate-qr.ts'
    },
    files: {
      'src/App.tsx': true,
      'src/main.tsx': true,
      'src/components/TicketCard.tsx': true,
      'src/components/QRScanner.tsx': true,
      'src/components/EventList.tsx': true,
      'src/components/ChainSelector.tsx': true,
      'src/lib/csv-adapter.ts': true,
      'src/lib/qr-utils.ts': true,
      'src/lib/ticket-store.ts': true,
      'src/pages/MyTickets.tsx': true,
      'src/pages/Scanner.tsx': true,
      'src/pages/Events.tsx': true,
      'src/types.ts': true,
      'index.html': true,
      'vite.config.ts': true,
      'tsconfig.json': true,
      'tailwind.config.js': true,
      '.env.example': true,
      'package.json': true
    },
    nextSteps: [
      'Start the dev server: npm run dev',
      'Open http://localhost:5173 in your mobile browser',
      'Create your first event and mint tickets',
      'Scan QR codes to validate tickets at the door',
      'Deploy to Vercel or Netlify for production'
    ]
  }
};

/**
 * Get all available template IDs
 * @returns {string[]} Array of template IDs
 */
function getTemplateIds() {
  return Object.keys(TEMPLATES);
}

/**
 * Get a template configuration by ID
 * @param {string} templateId - Template ID
 * @returns {object|null} Template configuration or null
 */
function getTemplate(templateId) {
  return TEMPLATES[templateId] || null;
}

/**
 * Get all template metadata with additional display info
 * @returns {Array} Array of template info objects
 */
function getAllTemplates() {
  return Object.values(TEMPLATES).map(t => ({
    id: t.id,
    name: t.name,
    description: t.description,
    category: t.category,
    framework: t.framework,
    defaultChains: t.defaultChains
  }));
}

/**
 * Get all available chain configurations
 * @returns {object} Chain configurations keyed by ID
 */
function getAllChains() {
  return CHAINS;
}

/**
 * Get chain configuration by ID
 * @param {string} chainId - Chain ID
 * @returns {object|null} Chain configuration or null
 */
function getChain(chainId) {
  return CHAINS[chainId] || null;
}

/**
 * Get chain IDs compatible with a template
 * @param {string} templateId - Template ID
 * @returns {string[]} Array of compatible chain IDs
 */
function getCompatibleChains(templateId) {
  const template = getTemplate(templateId);
  if (!template) return Object.keys(CHAINS);
  return template.defaultChains;
}

/**
 * Generate environment variable content for a template
 * @param {string} templateId - Template ID
 * @param {object} values - Environment variable values
 * @returns {string} .env file content
 */
function generateEnvContent(templateId, values = {}) {
  const template = getTemplate(templateId);
  if (!template) return '';

  let content = '# Environment Variables\n';
  content += '# Generated by create-csv-app\n';
  content += `# Date: ${new Date().toISOString()}\n\n`;

  for (const [key, config] of Object.entries(template.envVars)) {
    content += `# ${config.description}\n`;
    if (config.sensitive) {
      content += `# IMPORTANT: Never commit this value to version control\n`;
    }
    content += `${key}=${values[key] || config.default || ''}\n\n`;
  }

  return content;
}

/**
 * Generate .env.example content for a template
 * @param {string} templateId - Template ID
 * @returns {string} .env.example file content
 */
function generateEnvExampleContent(templateId) {
  const template = getTemplate(templateId);
  if (!template) return '';

  let content = '# Copy this file to .env and fill in your values\n';
  content += `# ${template.name} - Environment Variables\n\n`;

  for (const [key, config] of Object.entries(template.envVars)) {
    content += `# ${config.description}\n`;
    content += `${key}=${config.sensitive ? 'your_' + key.toLowerCase() : (config.default || '')}\n\n`;
  }

  return content;
}

module.exports = {
  TEMPLATES,
  CHAINS,
  getTemplateIds,
  getTemplate,
  getAllTemplates,
  getAllChains,
  getChain,
  getCompatibleChains,
  generateEnvContent,
  generateEnvExampleContent
};
