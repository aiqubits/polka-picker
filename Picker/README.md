# Picker - AI Smart System

[![GitHub license](https://img.shields.io/badge/license-Apache2.0-blue.svg)](https://github.com/aiqubits/picker/blob/main/LICENSE) [![Polkadot](https://img.shields.io/badge/Polkadot-Hub_Testnet-E6007A?logo=polkadot)](https://polkadot.network/) [![Rust-Agent](https://img.shields.io/badge/Rust%20Agent-0.04+-yellow)](https://tauri.app/) [![Tauri.app](https://img.shields.io/badge/Tauri-2.0+-yellow?logo=tauri)](https://tauri.app/) [![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](https://github.com/aiqubits/picker/pulls) [![Live Demo](https://img.shields.io/badge/demo-live-success)](https://www.openpick.org/)

## Introduction

### project name

Picker

### project creation date

2025-09-06

### project background

Next-generation Web3 decentralized internet-based AI reliability and availability smart system.

Picker aims to provide local PC work, entertainment, and life management functions without a backend server for all internet users (ready-to-use out of the box). It leverages artificial intelligence technology to enhance system intelligence and user experience. Meanwhile, it offers dual payment agent storage that supports Web3 wallets and point transactions, as well as a download market for all types of task Agents in multiple languages (Python, NodeJS, Shellscript...). Additionally, it provides high-performance on-chain operations (Transfer, Balance, Token, NFT, any contract call, etc.) and is fully compatible with the MCP protocol (different from Eliza Plugins).

Won one Conflux 2025 Summer Hackathon rewards before on 2025 September 30th.

### Core Problems and Motivations and Solutions

1. Reliable data privacy with fully transparent agents. There is no backend privacy server, and users have complete control of their data locally. Transactions are recorded on the blockchain, with a fully transparent process and no potential commercial costs.
   
2. The entire task lifecycle is controllable. Users can upload custom agent tasks or download agent tasks from the market. There is no hallucination during task execution, and users can fully trust the task execution results.
   
3. Ready-to-use. Users don't need to install any software. They only need to download the Picker application to start using it.

## Features planned for the Hackathon

### The status of project before participate the Hackathon

Only achieve compatibility with the Conflux Chain ecosystem and implement a basic user process.

https://github.com/conflux-fans/summerhackfest-2025/tree/main/projects/picker

### Features are planed for the Hackathon

1.Integrated and applied in Polka Revm, Including complete local validation testing.

2.Complete user data flow, including frontend calling custom smart contracts, frontend user wallet management, frontend Chatbot entrance calling AI management tasks, etc.

3.Example of complete implementation of memory, reflection, and mixed tool invocation for rust agent MCP server and client.

4.Example applications of on chain capability tools, such as balance, transfer, create token, create NFT, cross chain swap, etc.

5.Other details optimization.

## Architecture

### Diagram of architect for the project

![picker-arch.png](demo%2Fpicker-arch.png)

![picker-userflow.png](demo%2Fpicker-userflow.png)

###  Description for each components

## Schedule

- 2025-09-30: First v0.0.1 version Project creation, Only implement the simplest process, without including complete and innovative feature implementation.
- 2025-10-06: Fix desktop sidebar navigation and others.
- 2025-10-07: Impl AI api settings and chatbot task manager.
- 2025-10-11: Impl blockchain capability mcp server.
- 2025-10-20: Add Readme, Dos, PPT for Polka.
- 2025-10-23: Redefine the data types of custom contracts on-chain, and solidity contract tests, nodejs contract tests.
- 2025-10-24: Add api settings for Desktop Chatbot blockchain and other related optimizations.
- 2025-10-26: Complete the deployment and code verification of smart contract testing integration for the Polka passet-hub testnet.
- 2025-11-04: Impl rust-agent rust-agent medium to long-term memory, summary memory, automatic context size, session isolation For Picker.

### Smart Contract  

ERC20FactoryModule#ERC20Factory - 0x9BB37Ddf2f574b71C847F4659cBea7518fe172ee
https://blockscout-passet-hub.parity-testnet.parity.io/address/0x9BB37Ddf2f574b71C847F4659cBea7518fe172ee#code

ERC721FactoryModule#ERC721Factory - 0x9Dd5bCd24115E24774C43ee5811444AC57004D4f
https://blockscout-passet-hub.parity-testnet.parity.io/address/0x9BB37Ddf2f574b71C847F4659cBea7518fe172ee#code

PickerPaymentModule#PickerPayment - 0xc7a5983345b8577B0D27A7255e62A495A7AE8e7d
https://blockscout-passet-hub.parity-testnet.parity.io/address/0xc7a5983345b8577B0D27A7255e62A495A7AE8e7d#code

# Team info

- Team name: Picker
- Team members and their responsibilities:
    - Deporter - Technology - Project architecture, technical implementation, product optimization, and implementation of the AI Smart system
- Contact information (Email/Github hander/X): aiqubit@hotmail.com/aiqubits/ai_qubit

## Track and bounty

赛道一：构建下一代 Web3 生态系统

** mandatory before offline demo, submit aterial for Demo
1. Demo Video [https://youtu.be/l4-1wZfUMn8?si=nvCj9J9tYDu9lNf7]

<iframe width="80%" height="80%" src="https://www.youtube.com/embed/l4-1wZfUMn8?si=nvCj9J9tYDu9lNf7" title="YouTube video player" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share" referrerpolicy="strict-origin-when-cross-origin" allowfullscreen></iframe>


2. PPT [https://docs.google.com/presentation/d/1nKx2tJnDaQaKQiv4cXQ41uHVIB0YhGQT/edit?usp=sharing&ouid=105649966062078281366&rtpof=true&sd=true] 

https://picker-polka-ppt.vercel.app/1


## Key Modules

Rust Backend Server, User data flow, Web3 and Point payment, market cloud storage hosting, Integration:

[Code location](./server)

Second level authorization payment contract, and ERC Factory contract:

[Code location](./contract)

Customize AI Agent for Web3:

[Code location](./rust-agent-crate)

Desktop client, Local tasks, Chatbot, Market entrance, user page, contract call entrance:

[Code location](./desktop)

Chain capability mcp server, transfer, balance, publish token, issue nft, cross chain swap:

[Code location](./chain-capability-mcp-server)

Example apps, agent task template:

[Code location](./apps)

- Frontend: Tauri2.0 + React19 + TypeScript + Vite, desktop application, providing user interfaces, interaction logic, etc.
- Backend: Rust Axum + Sqlite3, implementing a dual transaction system (Web3 wallet, point transactions), user authentication, and functions such as downloading and uploading from the agents cloud storage market.
- Smart Contracts: Solidity + Hardhat3, implementing second - level authorization payment contracts, agents data verification, fund management, etc.
- AI Agent Framework: Rust + Tokio + OpenAI Compatible API, implementing a high - performance Web3 AI Agent framework, tool calling (MCP protocol), ReAct (Reasoning and Acting), conversation memory, and integration of large language model interfaces.
- Example Apps: Example applications in Node.js, Python, PowerShell, Bash shell, etc., demonstrating framework usage and providing development templates.
- On-chain Capability MCP Server: Implementing on-chain operations (Transfer Token, Check Balance, Issue Token, Issue NFT, Cross-chain Swap, etc.), fully compatible with the MCP protocol.

## Minimum Reproduction Script

```Solidity
# deploy contract
npx hardhat ignition deploy --network customize ignition/modules/PickerPayment.ts
npx hardhat ignition deploy --network customize ignition/modules/ERC20Factory.ts
npx hardhat ignition deploy --network customize ignition/modules/ERC721Factory.ts

# running and testing
npx hardhat test solidity
npx hardhat test nodejs
```

## Run & Reproduce

- Prerequisites: Node 22+, npm, Git, Cargo Rust 1.89+, Tauri2.0, Hardhat 3.0, OpenAI compatible API URL and Key
- Environment Variables Sample:

backend: ./server/config.toml

frontend: Default values are already built into the code

```
api_base_url = "http://127.0.0.1:3000"
request_timeout_ms = 30000
max_retries = 3
ai_api_url = "https://api.deepseek.com/v1"
ai_api_key = ""
ai_model = "deepseek-chat"

[blockchain]
rpc_url = "https://testnet-passet-hub-eth-rpc.polkadot.io"
explorer_url = "https://blockscout-passet-hub.parity-testnet.parity.io"
wallet_private_key = ""
token_usdt_url = "https://www.okx.com/api/v5/market/ticker?instId=DOT-USDT"
# sepolia cross chain pay
usdt_contract_address = "0xd53e9530107a8d8856099d7d80126478d48e06dA"
meson_contract_address = "0x0d12d15b26a32e72A3330B2ac9016A22b1410CB6"
erc20_factory_address = "0x9712C7792fF62373f4ddBeE53DBf9BeCB63D80dB"
erc721_factory_address = "0xDc49Fe683D54Ee2E37459b4615DebA8dbee3cB9A"
```

- One-click Start (Local Example):

```PowerShell
# Start the backend service
cd server
cargo run

# Start the mcp server
cd chain-capability-mcp-server
cargo run

# Start the client application
cd desktop
npx tauri dev
```

- Account and Test Instructions:

Username: testdata@openpick.org

Password: testpassword


# Roadmap & Impact

- 1-3 weeks after the hackathon: The PC store can be launched immediately or a self-built official website can be set up. The official website has been completed at https://www.openpick.org/
- 1-3 months after the hackathon: Improve the registration mechanism of the task market, optimize the AI Agent framework, complete more example applications of agent tasks, and provide more on-chain contract capability interfaces
- Expected value to the Polkadot Smart Contract APP ecosystem: Make Polkadot Smart Contract APP truly integrate into the daily work, life, and entertainment of ordinary internet users
