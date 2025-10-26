import type { HardhatUserConfig } from "hardhat/config";
import hardhatVerifyPlugin from "@nomicfoundation/hardhat-verify";
import hardhatToolboxViemPlugin from "@nomicfoundation/hardhat-toolbox-viem";
import { configVariable } from "hardhat/config";

const config: HardhatUserConfig = {
  plugins: [hardhatToolboxViemPlugin, hardhatVerifyPlugin],
  solidity: {
        version: "0.8.28",
        settings: {
          optimizer: {
            enabled: true,
            runs: 200,
          },
        },
  },
  networks: {
    hardhatMainnet: {
      type: "edr-simulated",
      chainType: "l1",
    },
    hardhatOp: {
      type: "edr-simulated",
      chainType: "op",
    },
    "passet-hub": {
      type: "http",
      chainType: "l1",
      url: 'https://testnet-passet-hub-eth-rpc.polkadot.io',
      // url: 'https://blockscout-passet-hub.parity-testnet.parity.io/api/eth-rpc',
      accounts: [configVariable("SEPOLIA_PRIVATE_KEY")],
    },    
    customize: {
      type: "http",
      chainType: "l1",
      url: configVariable("SEPOLIA_RPC_URL"),
      accounts: [configVariable("SEPOLIA_PRIVATE_KEY")],
    },
  },
  chainDescriptors: {
    420420422: {
      name: "passet-hub",
      chainType: "l1",
      blockExplorers: {
        etherscan: {
          name: "passet-hub",
          url: "https://blockscout-passet-hub.parity-testnet.parity.io",
          apiUrl: "https://blockscout-passet-hub.parity-testnet.parity.io/api",
        },
      },
    },
  },
  verify:{
    etherscan: {
      apiKey: 'empty',
    }
  }  
  // chainDescriptors: {
  //   11155111: {
  //     name: "Sepolia",
  //     chainType: "l1",
  //     blockExplorers: {
  //       etherscan: {
  //         name: "Sepolia Explorer",
  //         url: "https://sepolia.etherscan.io/",
  //         apiUrl: "https://api.etherscan.io/v2/api",
  //       },
  //     },
  //   },
  // },
  // verify:{
  //   etherscan: {
  //     apiKey: configVariable("ETHERSCAN_API_KEY"),
  //   }
  // }
};

export default config;
