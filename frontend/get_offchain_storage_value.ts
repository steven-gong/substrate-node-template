import { ApiPromise, WsProvider } from '@polkadot/api';
import { u8aToString } from "@polkadot/util";

async function read_offchain_storage() {
    const provider = new WsProvider('ws://127.0.0.1:9944');
    const api = await ApiPromise.create({ provider });

    const ONCHAIN_TX_KEY = "template_pallet::indexing1";

    const response = await api.rpc.offchain.localStorageGet("PERSISTENT", ONCHAIN_TX_KEY);
    const data_in_hex = response.toString();
    console.log(`hexValue ${data_in_hex}`);

    const data_in_u8a = hexToUint8Array(data_in_hex);
    console.log(`indexing_data ${data_in_u8a}`);

    const data_str = u8aToString(data_in_u8a);
    console.log(`indexing_data ${u8aToString(data_in_u8a)}`);

    // TODO: decode the IndexingData
}

function hexToUint8Array(hex: string): Uint8Array {
    const bytes = new Uint8Array(hex.length / 2);

    for (let i = 0; i < hex.length; i += 2) {
        const byte = parseInt(hex.slice(i, i + 2), 16);
        bytes[i / 2] = byte;
    }

    return bytes;
}

read_offchain_storage().catch(console.error);