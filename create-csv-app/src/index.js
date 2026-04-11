#!/usr/bin/env node
/**
 * create-csv-app — Scaffolds a CSV Adapter project in 5 minutes
 *
 * Usage:
 *   npx create-csv-app@latest [project-name] [options]
 *   npx create-csv-app@latest --help
 *
 * Options:
 *   --template <name>    Template to use (basic-transfer, nft-marketplace, etc.)
 *   --chain <chains>     Comma-separated chains to include
 *   --language <lang>    TypeScript (default) or JavaScript
 *   --package-manager    npm, yarn, pnpm, or bun
 */

const { Command } = require('commander');
const prompts = require('prompts');
const chalk = require('chalk');
const ora = require('ora');
const path = require('path');
const fs = require('fs-extra');
const degit = require('degit');
const {
  getAllTemplates,
  getTemplate,
  getAllChains,
  getCompatibleChains,
  generateEnvContent,
  generateEnvExampleContent
} = require('./config');
const {
  validateProjectName,
  checkNodeVersion,
  checkGitInstalled,
  installDependencies,
  generateMnemonic
} = require('./utils');

const program = new Command();

program
  .name('create-csv-app')
  .description('Create a cross-chain CSV Adapter project in 5 minutes')
  .version('0.1.0')
  .argument('[project-name]', 'Name of your project')
  .option('--template <name>', 'Template to use')
  .option('--chain <chains>', 'Comma-separated list of chains')
  .option('--language <lang>', 'TypeScript or JavaScript', 'typescript')
  .option('--package-manager <pm>', 'Package manager to use', 'npm')
  .option('--no-install', 'Skip dependency installation')
  .option('--yes', 'Skip prompts and use defaults')
  .action(main);

async function main(projectName, options) {
  console.log(chalk.bold('\n🚀 CSV Adapter — Create Cross-Chain App\n'));

  // Pre-flight checks
  checkNodeVersion();
  const gitOk = checkGitInstalled();
  if (!gitOk) {
    console.warn(chalk.yellow('⚠  Git not found. Some features may not work.'));
  }

  // Determine project name
  let name = projectName;
  if (!name) {
    if (options.yes) {
      name = 'my-csv-app';
    } else {
      const { projectName: answer } = await prompts({
        type: 'text',
        name: 'projectName',
        message: 'Project name:',
        initial: 'my-csv-app',
        validate: (v) => validateProjectName(v)
      });
      name = answer || 'my-csv-app';
    }
  }

  // Validate project name
  const validationError = validateProjectName(name);
  if (validationError !== true) {
    console.error(chalk.red(`\n✗ Invalid project name: ${validationError}`));
    process.exit(1);
  }

  const targetDir = path.resolve(process.cwd(), name);
  if (await fs.pathExists(targetDir)) {
    const contents = await fs.readdir(targetDir);
    if (contents.length > 0) {
      const { proceed } = await prompts({
        type: 'confirm',
        name: 'proceed',
        message: `Directory ${name} is not empty. Continue anyway?`,
        initial: false
      });
      if (!proceed) {
        console.log(chalk.yellow('\nAborted.'));
        process.exit(0);
      }
    }
  }

  // Select template
  let templateId = options.template;
  if (!templateId) {
    if (options.yes) {
      templateId = 'basic-transfer';
    } else {
      const templates = getAllTemplates();
      const { template: selected } = await prompts({
        type: 'select',
        name: 'template',
        message: 'What do you want to build?',
        choices: templates.map(t => ({
          title: chalk.bold(t.name),
          description: t.description,
          value: t.id
        })),
        initial: 0
      });
      templateId = selected || 'basic-transfer';
    }
  }

  const template = getTemplate(templateId);
  if (!template) {
    console.error(chalk.red(`\n✗ Unknown template: ${templateId}`));
    console.log(`\nAvailable templates: ${getAllTemplates().map(t => t.id).join(', ')}`);
    process.exit(1);
  }

  // Select chains
  let chains = options.chain ? options.chain.split(',').map(c => c.trim()) : null;
  if (!chains) {
    const compatibleChains = getCompatibleChains(templateId);
    const allChains = Object.values(getAllChains());

    if (options.yes) {
      chains = compatibleChains;
    } else {
      const { selectedChains } = await prompts({
        type: 'multiselect',
        name: 'selectedChains',
        message: 'Which chains do you want to include?',
        choices: allChains.map(c => ({
          title: chalk.bold(c.name),
          description: c.testnetName,
          value: c.id,
          selected: compatibleChains.includes(c.id)
        })),
        min: 1
      });
      chains = selectedChains || compatibleChains;
    }
  }

  // Package manager
  let packageManager = options.packageManager;
  if (!options.yes && !options.packageManager) {
    const { pm } = await prompts({
      type: 'select',
      name: 'pm',
      message: 'Package manager:',
      choices: [
        { title: 'npm', value: 'npm' },
        { title: 'yarn', value: 'yarn' },
        { title: 'pnpm', value: 'pnpm' },
        { title: 'bun', value: 'bun' }
      ],
      initial: 0
    });
    packageManager = pm || 'npm';
  }

  // Summary
  console.log(chalk.bold('\n📋 Project Summary\n'));
  console.log(`  Name:           ${chalk.cyan(name)}`);
  console.log(`  Template:       ${chalk.cyan(template.name)}`);
  console.log(`  Chains:         ${chains.map(c => chalk.cyan(c)).join(', ')}`);
  console.log(`  Package Manager: ${chalk.cyan(packageManager)}`);
  console.log(`  Directory:      ${chalk.cyan(targetDir)}\n`);

  if (!options.yes) {
    const { confirmed } = await prompts({
      type: 'confirm',
      name: 'confirmed',
      message: 'Create project with these settings?',
      initial: true
    });
    if (!confirmed) {
      console.log(chalk.yellow('\nAborted.'));
      process.exit(0);
    }
  }

  // Start scaffolding
  const spinner = ora('Creating project...').start();

  try {
    // Create directory
    await fs.ensureDir(targetDir);

    // Generate mnemonic
    const mnemonic = generateMnemonic();

    // Generate .env
    const envVars = { MNEMONIC: mnemonic };
    chains.forEach(chainId => {
      const chain = getAllChains()[chainId];
      if (chain) {
        envVars[chain.rpcEnvVar] = chain.defaultTestnetRpc;
      }
    });

    // Write .env
    const envContent = generateEnvContent(templateId, envVars);
    await fs.writeFile(path.join(targetDir, '.env'), envContent);

    // Write .env.example
    const envExample = generateEnvExampleContent(templateId);
    await fs.writeFile(path.join(targetDir, '.env.example'), envExample);

    // Write .gitignore
    const gitignore = `# Dependencies
node_modules/

# Environment
.env

# Build
dist/
.next/
out/

# IDE
.vscode/
.idea/
*.swp

# OS
.DS_Store

# CSV
data/
*.db
`;
    await fs.writeFile(path.join(targetDir, '.gitignore'), gitignore);

    // Generate package.json
    const pkgJson = {
      name,
      version: '0.1.0',
      private: true,
      description: `${template.name} built with CSV Adapter`,
      scripts: template.scripts,
      dependencies: {
        ...template.dependencies
      },
      devDependencies: {
        typescript: '^5.3.0',
        tsx: '^4.7.0',
        ...(template.language !== 'typescript' ? {} : {})
      }
    };

    await fs.writeJson(path.join(targetDir, 'package.json'), pkgJson, { spaces: 2 });

    // Generate README
    const readme = `# ${template.name}

Built with [CSV Adapter](https://github.com/zorvan/csv-adapter)

## Getting Started

\`\`\`bash
# Install dependencies
${packageManager} install

# Start development server
${packageManager} run dev
\`\`\`

## Next Steps

${template.nextSteps.map((step, i) => `${i + 1}. ${step}`).join('\n')}

## Configuration

Edit \`.env\` to configure your RPC endpoints and wallet.

**IMPORTANT:** Never commit \`.env\` to version control.

## Documentation

- [CSV Adapter Docs](https://docs.csv.dev)
- [SDK Reference](https://docs.csv.dev/api)
- [Examples](https://examples.csv.dev)

## License

MIT
`;
    await fs.writeFile(path.join(targetDir, 'README.md'), readme);

    // Generate basic source file
    const srcDir = path.join(targetDir, 'src');
    await fs.ensureDir(srcDir);

    const indexTs = `/**
 * ${template.name}
 * Generated by create-csv-app
 */

import { CSV, Chain } from '@csv-adapter/sdk';

async function main() {
  // Initialize CSV client
  const csv = await CSV.fromMnemonic(process.env.MNEMONIC!, {
    chains: [${chains.map(c => `Chain.${c.charAt(0).toUpperCase() + c.slice(1)}`).join(', ')}],
    network: 'testnet'
  });

  console.log('✅ CSV client initialized');
  console.log('📝 Chains:', ${JSON.stringify(chains)});

  // Check balances
  for (const chain of ${JSON.stringify(chains)}) {
    try {
      const balance = await csv.wallet.getBalance(chain);
      console.log(\`💰 \${chain}: \${balance.amount} \${balance.currency}\`);
    } catch (err) {
      console.log(\`⚠️  \${chain}: Could not fetch balance\`);
    }
  }

  console.log('\\n🎉 Ready to build!');
  console.log('📖 Next: Edit src/index.ts to add your cross-chain logic');
}

main().catch(console.error);
`;
    await fs.writeFile(path.join(srcDir, 'index.ts'), indexTs);

    // Generate tsconfig.json
    const tsconfig = {
      compilerOptions: {
        target: 'ES2022',
        module: 'ESNext',
        moduleResolution: 'bundler',
        lib: ['ES2022'],
        outDir: './dist',
        rootDir: './src',
        strict: true,
        esModuleInterop: true,
        skipLibCheck: true,
        forceConsistentCasingInFileNames: true,
        resolveJsonModule: true
      },
      include: ['src/**/*'],
      exclude: ['node_modules', 'dist']
    };
    await fs.writeJson(path.join(targetDir, 'tsconfig.json'), tsconfig, { spaces: 2 });

    spinner.succeed('Project created');

    // Install dependencies
    if (options.install !== false) {
      const installSpinner = ora('Installing dependencies...').start();
      try {
        await installDependencies(targetDir, packageManager);
        installSpinner.succeed('Dependencies installed');
      } catch (err) {
        installSpinner.warn('Dependency installation failed. Run manually:');
        console.log(chalk.yellow(`  cd ${name} && ${packageManager} install`));
      }
    }

    // Success message
    console.log(chalk.bold.green('\n🎉 Project created successfully!\n'));
    console.log(chalk.bold('📁 Next steps:'));
    console.log(`   ${chalk.cyan(`cd ${name}`)}`);
    console.log(`   ${chalk.cyan(`${packageManager} run dev`)}\n`);

    console.log(chalk.bold('🔑 Your dev wallet mnemonic:'));
    console.log(chalk.yellow(`   ${mnemonic}`));
    console.log(chalk.yellow('   ⚠️  Save this! You\'ll need it for testnet funds.\n'));

    console.log(chalk.bold('📖 Resources:'));
    console.log('   Docs:     https://docs.csv.dev');
    console.log('   Examples: https://examples.csv.dev');
    console.log('   Discord:  https://discord.gg/csv-adapter\n');

    console.log(chalk.bold('🏆 Tutorial:'));
    console.log(`   ${chalk.cyan('csv tutorial cross-chain-basics')}\n`);

  } catch (error) {
    spinner.fail('Project creation failed');
    console.error(chalk.red(error.message));
    process.exit(1);
  }
}

program.parse();
