import './App.css'
import { useEffect, useState, useMemo } from 'react'
import {
  useWallet,
  useConnectedWallet,
  useLCDClient,
  WalletStatus,
} from '@terra-money/wallet-provider'
import { contractAddress } from './contract/address'
import { DistributeClient } from './contract/clients/DistributeClient'
import { ConnectWallet } from './components/ConnectWallet'
import { MsgSend, MnemonicKey, Coins, LCDClient } from "@terra-money/terra.js";
const App = () => {
  // const [count, setCount] = useState(0)
  // const [updating, setUpdating] = useState(true)
  // const [resetValue, setResetValue] = useState(0)

  const { status, network, wallets, availableConnectTypes, availableConnections, connect, disconnect } = useWallet();
  const connectedWallet = useConnectedWallet()
  const lcd = useLCDClient()

  const contractClient = useMemo(() => {
    if (!connectedWallet) {
      return
    }
    return new DistributeClient(lcd, connectedWallet, contractAddress('distribute', connectedWallet),)
  }, [lcd, connectedWallet])

  const connectWallet = async () => {
    try {
      await connect(availableConnectTypes[0]);
      console.log("succesfully connected")
    } catch (error) {
      console.log(error)
    }
  }
  const disconnectWallet = async () => {
    try {
      await disconnect();
      console.log("succesfully disconnected")
    } catch (error) {
      console.log(error)
    }

  }
  const walletStatus = async () => {
    console.log(status);
  }
  const withdrawTransaction = async (address: string) => {
    try {
      if (status == "WALLET_CONNECTED") {
        if (contractClient) {
          await contractClient.withdraw({ receiver: address });
        }
      } else console.log("Wallet not connected")
    } catch (error) {
      console.log(error)
    }
  }
  const depositTransaction = async () => {
    try {
      if (status == "WALLET_CONNECTED") {
        if (contractClient) {
          await contractClient.deposit();
        }
      } else console.log("Wallet not connected")
    } catch (error) {
      console.log(error)
    }
  }
  const setLimit = async (amount: string) => {
    try {
      if (status == "WALLET_CONNECTED") {
        if (contractClient) {
          await contractClient.set({ amount: amount });
        }
      } else console.log("Wallet not connected")
    } catch (error) {
      console.log(error)
    }
  }
  const queryContract = async () => {
    try {
      const gasPrices = await (
        await fetch("https://pisco-api.terra.dev/gas-prices", { redirect: 'follow' })
      ).json();
      const gasPricesCoins = new Coins(gasPrices);

      const lcd = new LCDClient({
        URL: "https://pisco-lcd.terra.dev/",
        chainID: "pisco-1",
        gasPrices: gasPricesCoins,
        gasAdjustment: "1.5",
        // gas: "10000000",
        isClassic: false, // optional parameter, false by default
      });
      const ContractAddress = "";
      const response = await lcd.wasm.contractQuery(
        ContractAddress, {
        records: {},
      });
      console.log(response);
    } catch (error) {
      console.log(error)
    }
  }

  return (
    <div className="App">
      <section>
        <pre>
          {JSON.stringify(
            {
              status,
              network,
              wallets,
              availableConnectTypes,
            },
            null,
            2,
          )}
        </pre>
      </section>
      <header className="App-header">
        <div style={{ display: 'inline' }}>
          <p>Wallet connect</p>
        </div>
        {status === WalletStatus.WALLET_CONNECTED && (<button type="button" onClick={() => withdrawTransaction("terra1s628arrdq2nrkmvdj4l66zjy5z2r3flkfxswne")}>Withdraw</button>
        )}
        {status === WalletStatus.WALLET_CONNECTED && (<button type="button" onClick={depositTransaction}>Deposit</button>
        )}
        {status === WalletStatus.WALLET_CONNECTED && (<button type="button" onClick={() => setLimit("2000000")}>setLimit</button>
        )}
      </header>
      <button onClick={connectWallet}>connect</button>
      <button onClick={disconnectWallet}>Disconnect</button>
      <button onClick={walletStatus}>Wallet-status</button>
      <button onClick={queryContract}>Query</button>
    </div>

  )
}

export default App