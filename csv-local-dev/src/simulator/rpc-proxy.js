#!/usr/bin/env node
'use strict';

const crypto = require('crypto');

/**
 * Unified RPC Proxy
 * 
 * Routes requests to correct chain simulators, handles request/response logging,
 * rate limiting, and error injection for testing failures.
 */
class RpcProxy {
  constructor(options = {}) {
    this.simulators = options.simulators || {};
    this.registry = options.registry;
    
    // Logging
    this.requestLog = [];
    this.maxLogSize = 1000;

    // Rate limiting
    this.rateLimit = new Map(); // ip -> { count, resetTime }
    this.rateLimitMax = 100; // requests per minute
    this.rateLimitWindow = 60000; // 1 minute

    // Error injection (for testing failures)
    this.errorInjection = {
      enabled: false,
      chains: {}, // chain -> { errorRate, errorTypes }
      delayMs: 0
    };

    // Statistics
    this.stats = {
      totalRequests: 0,
      requestsByChain: {},
      errorsByChain: {},
      avgLatency: 0,
      totalLatency: 0
    };
  }

  /**
   * Setup Express routes
   */
  setupRoutes(app) {
    // Catch-all for RPC routing
    app.post('/rpc/:chain?', async (req, res) => {
      const chain = req.params.chain;
      if (!chain || !this.simulators[chain]) {
        return res.status(400).json({
          jsonrpc: '2.0',
          error: { code: -32601, message: `Chain not found: ${chain}` },
          id: req.body?.id || null
        });
      }

      try {
        const result = await this.routeRequest(chain, req.body, req.ip);
        res.json(result);
      } catch (error) {
        res.status(500).json({
          jsonrpc: '2.0',
          error: { code: -32000, message: error.message },
          id: req.body?.id || null
        });
      }
    });

    // Shutdown endpoint
    app.post('/shutdown', async (req, res) => {
      res.json({ status: 'shutting_down' });
      // Delay shutdown to allow response to be sent
      setTimeout(() => {
        process.exit(0);
      }, 100);
    });

    // Error injection endpoints
    app.post('/debug/inject-error', (req, res) => {
      const { chain, errorRate, errorType, delayMs } = req.body;
      this.configureErrorInjection({ chain, errorRate, errorType, delayMs });
      res.json({ status: 'configured', config: this.errorInjection });
    });

    app.delete('/debug/inject-error', (req, res) => {
      this.errorInjection.enabled = false;
      this.errorInjection.delayMs = 0;
      res.json({ status: 'disabled' });
    });

    // Rate limit config
    app.post('/debug/rate-limit', (req, res) => {
      const { maxRequests, windowMs } = req.body;
      if (maxRequests) this.rateLimitMax = maxRequests;
      if (windowMs) this.rateLimitWindow = windowMs;
      res.json({ rateLimitMax: this.rateLimitMax, rateLimitWindow: this.rateLimitWindow });
    });

    // Request log
    app.get('/debug/requests', (req, res) => {
      const limit = parseInt(req.query.limit, 10) || 100;
      res.json(this.requestLog.slice(-limit));
    });

    // Stats
    app.get('/debug/stats', (req, res) => {
      res.json({
        ...this.stats,
        avgLatency: this.stats.totalRequests > 0 
          ? this.stats.totalLatency / this.stats.totalRequests 
          : 0
      });
    });
  }

  /**
   * Route a request to the appropriate chain simulator
   */
  async routeRequest(chain, body, ip) {
    const startTime = Date.now();

    // Check rate limit
    if (!this.checkRateLimit(ip)) {
      throw new Error('Rate limit exceeded');
    }

    // Check error injection
    if (this.shouldInjectError(chain)) {
      this.stats.errorsByChain[chain] = (this.stats.errorsByChain[chain] || 0) + 1;
      throw new Error(this.getInjectedError());
    }

    // Add artificial delay if configured
    if (this.errorInjection.delayMs > 0) {
      await new Promise(resolve => setTimeout(resolve, this.errorInjection.delayMs));
    }

    // Route to simulator
    const simulator = this.simulators[chain];
    if (!simulator) {
      throw new Error(`Simulator not found for chain: ${chain}`);
    }

    const result = await simulator.handleRpc(body);

    // Log request
    const latency = Date.now() - startTime;
    this.logRequest(chain, body, result, latency);

    // Update stats
    this.stats.totalRequests++;
    this.stats.requestsByChain[chain] = (this.stats.requestsByChain[chain] || 0) + 1;
    this.stats.totalLatency += latency;

    return result;
  }

  /**
   * Check rate limit for an IP
   */
  checkRateLimit(ip) {
    const now = Date.now();
    const client = this.rateLimit.get(ip);

    if (!client || now > client.resetTime) {
      this.rateLimit.set(ip, { count: 1, resetTime: now + this.rateLimitWindow });
      return true;
    }

    client.count++;
    return client.count <= this.rateLimitMax;
  }

  /**
   * Configure error injection
   */
  configureErrorInjection({ chain, errorRate, errorType, delayMs }) {
    this.errorInjection.enabled = true;
    
    if (chain) {
      this.errorInjection.chains[chain] = {
        errorRate: errorRate || 0.1,
        errorType: errorType || 'timeout'
      };
    }

    if (delayMs !== undefined) {
      this.errorInjection.delayMs = delayMs;
    }
  }

  /**
   * Check if error should be injected
   */
  shouldInjectError(chain) {
    if (!this.errorInjection.enabled) return false;
    const chainConfig = this.errorInjection.chains[chain];
    if (!chainConfig) return false;

    return Math.random() < chainConfig.errorRate;
  }

  /**
   * Get a random injected error
   */
  getInjectedError() {
    const errors = [
      'Connection timeout',
      'Internal server error',
      'Invalid response',
      'Chain reorg detected',
      'Transaction pool full',
      'Rate limit exceeded',
      'Service temporarily unavailable'
    ];
    return errors[Math.floor(Math.random() * errors.length)];
  }

  /**
   * Log a request
   */
  logRequest(chain, request, response, latency) {
    const entry = {
      timestamp: new Date().toISOString(),
      chain,
      method: request.method || 'unknown',
      latency,
      success: !response.error && !response.result?.error
    };

    this.requestLog.push(entry);
    if (this.requestLog.length > this.maxLogSize) {
      this.requestLog.shift();
    }
  }

  /**
   * Get dashboard data
   */
  getDashboardData() {
    const data = {
      chains: {},
      wallets: {},
      registry: {
        totalRights: 0,
        activeTransfers: 0,
        completedTransfers: 0
      },
      stats: {
        totalRequests: this.stats.totalRequests,
        requestsByChain: this.stats.requestsByChain,
        avgLatency: this.stats.totalRequests > 0 
          ? Math.round(this.stats.totalLatency / this.stats.totalRequests) 
          : 0
      }
    };

    // Chain data
    for (const [name, simulator] of Object.entries(this.simulators)) {
      data.chains[name] = simulator.getStatus();
    }

    // Registry data
    if (this.registry) {
      const registryStats = this.registry.getStats();
      data.registry.totalRights = registryStats.totalRightsCreated;
      data.registry.activeTransfers = registryStats.pendingTransfers;
      data.registry.completedTransfers = registryStats.totalTransfersCompleted;
    }

    return data;
  }

  /**
   * Clear request log
   */
  clearLog() {
    this.requestLog = [];
  }

  /**
   * Reset statistics
   */
  resetStats() {
    this.stats = {
      totalRequests: 0,
      requestsByChain: {},
      errorsByChain: {},
      avgLatency: 0,
      totalLatency: 0
    };
    this.rateLimit.clear();
    this.requestLog = [];
  }
}

module.exports = { RpcProxy };
