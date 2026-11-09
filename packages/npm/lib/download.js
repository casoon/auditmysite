'use strict';

const http = require('node:http');
const https = require('node:https');
const tls = require('node:tls');

const MAX_REDIRECTS = 5;

function proxyFor(env) {
  const value = env.HTTPS_PROXY || env.https_proxy;
  if (!value) return null;
  const proxy = new URL(value);
  if (proxy.protocol !== 'http:') {
    throw new Error(`Unsupported proxy protocol ${proxy.protocol} (only http:// proxies are supported)`);
  }
  return proxy;
}

// HTTPS through an HTTP proxy: CONNECT tunnel, then TLS to the real host.
class TunnelAgent extends https.Agent {
  constructor(proxy) {
    super();
    this.proxy = proxy;
  }

  createConnection(options, callback) {
    const headers = { Host: `${options.host}:${options.port}` };
    if (this.proxy.username) {
      const credentials = `${decodeURIComponent(this.proxy.username)}:${decodeURIComponent(this.proxy.password)}`;
      headers['Proxy-Authorization'] = `Basic ${Buffer.from(credentials).toString('base64')}`;
    }
    const request = http.request({
      host: this.proxy.hostname,
      port: this.proxy.port || 80,
      method: 'CONNECT',
      path: `${options.host}:${options.port}`,
      headers,
    });
    request.once('connect', (response, socket) => {
      if (response.statusCode !== 200) {
        socket.destroy();
        callback(new Error(`Proxy refused CONNECT: HTTP ${response.statusCode}`));
        return;
      }
      callback(null, tls.connect({ socket, servername: options.host }));
    });
    request.once('error', callback);
    request.end();
  }
}

function get(url, env = process.env, redirects = 0) {
  return new Promise((resolve, reject) => {
    if (!url.startsWith('https://')) {
      reject(new Error(`Refusing non-HTTPS URL: ${url}`));
      return;
    }
    const proxy = proxyFor(env);
    const request = https.get(url, proxy ? { agent: new TunnelAgent(proxy) } : {}, (response) => {
      const { statusCode, headers } = response;
      if (statusCode >= 300 && statusCode < 400 && headers.location) {
        response.resume();
        if (redirects >= MAX_REDIRECTS) {
          reject(new Error(`Too many redirects fetching ${url}`));
          return;
        }
        get(new URL(headers.location, url).href, env, redirects + 1).then(resolve, reject);
        return;
      }
      if (statusCode !== 200) {
        response.resume();
        reject(new Error(`Download failed: HTTP ${statusCode} for ${url}`));
        return;
      }
      const chunks = [];
      response.on('data', (chunk) => chunks.push(chunk));
      response.on('end', () => resolve(Buffer.concat(chunks)));
      response.on('error', reject);
    });
    request.on('error', reject);
  });
}

module.exports = { get, proxyFor };
