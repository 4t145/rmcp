import { Client } from "@modelcontextprotocol/sdk/client/index.js";
import { SSEClientTransport } from "@modelcontextprotocol/sdk/client/sse.js";

const transport = new SSEClientTransport( new URL(`http://127.0.0.1:8000/sse`));

const client = new Client(
  {
    name: "example-client",
    version: "1.0.0"
  },
  {
    capabilities: {
      prompts: {},
      resources: {},
      tools: {}
    }
  }
);
await client.connect(transport);
const tools = await client.listTools();
console.log(tools);
const resources = await client.listResources();
console.log(resources);
const templates = await client.listResourceTemplates();
console.log(templates);
const prompts = await client.listPrompts();
console.log(prompts);
await client.close();
