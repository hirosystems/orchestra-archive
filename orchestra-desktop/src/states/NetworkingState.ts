import { createSlice, PayloadAction } from "@reduxjs/toolkit";
import { RootState } from "../stores/root";
import { ClarityAbiType, Contract } from "../types";
import { BitcoinBlockMetadata, Block, BlockIdentifier, StacksDataMapDeleteEventData, StacksDataMapInsertEventData, StacksDataMapUpdateEventData, StacksDataVarSetEventData, StacksNFTBurnEventData, StacksNFTMintEventData, StacksNFTTransferEventData, StacksFTBurnEventData, StacksFTMintEventData, StacksFTTransferEventData, StacksTransactionEventType, TransactionIdentifier, StacksBlock, BitcoinBlock } from "../types/clarinet";

export enum ActiveFeature {
  StateExplorer,
}

export type StateExplorerStateUpdateWatch = Record<"StateExplorerWatch", StateExplorerStateUpdateWatchData>
export type StateExplorerStateUpdateInit = Record<"StateExplorerInitialization", StateExplorerStateUpdateInitData>
export type BootNetwork = Record<"BootNetwork", BootNetworkData>


export interface StateExplorerStateUpdate {
  update: StateExplorerStateUpdateWatch | StateExplorerStateUpdateInit | BootNetwork;
}

export interface StateExplorerStateUpdateInitData {
  contracts: Array<Contract>;
}

export type VarValues = Record<"Var", VarValuesData>
export type MapValues = Record<"Map", MapValuesData>
export type NftValues = Record<"Nft", NftValuesData>
export type FtValues = Record<"Ft", FtValuesData>

export interface StateExplorerStateUpdateWatchData {
  stacks_blocks: Array<StacksBlock>;
  bitcoin_blocks: Array<BitcoinBlock>;
  contract_identifier: string;
  field_name: string;
  field_values: VarValues | MapValues | NftValues | FtValues;
}

export interface BootNetworkData {
  status: string;
  bitcoin_chain_height: number,
  stacks_chain_height: number,
  protocol_deployed: boolean,
  contracts: Array<Contract>,
  protocol_id: number,
  protocol_name: string,
}

export enum StateExplorerState {
  None = "None",
  BootNetwork = "BootNetwork",
  Initialization = "StateExplorerInitialization",
  Sleep = "StateExplorerSleep",
  Watch = "StateExplorerWatch",
}

export interface Request {
  protocol_id: number;
  request: any;
}

export interface BootNetworkState {
  manifest_path: string;
}

export interface StateExplorerInitializationState {
  manifest_path: string;
}

export interface StateExplorerPauseState {}

export type WatchedTarget = Record<
  TargetType,
  ContractFieldTarget | WalletTarget // | ContractTarget
>;

// export interface ContractTarget {
//   contract_identifier: string;
// }

// export interface ContractTargetUpdate {
//   contract_identifier: string;
// }

export interface ContractFieldTarget {
  contract_identifier: string;
  field_name: string;
}

export interface ContractFieldTargetUpdate {
  contract_identifier: string;
  field_name: string;
}

export enum FieldValues {
  Var = "Var",
  Map = "Map",
  FT = "Ft",
  NFT = "Nft",
}

export type VarSetEvent = Record<StacksTransactionEventType.StacksDataVarSetEvent, StacksDataVarSetEventData>
export type MapInsertEvent = Record<StacksTransactionEventType.StacksDataMapInsertEvent, StacksDataMapInsertEventData>
export type MapUpdateEvent = Record<StacksTransactionEventType.StacksDataMapUpdateEvent, StacksDataMapUpdateEventData>
export type MapDeleteEvent = Record<StacksTransactionEventType.StacksDataMapDeleteEvent, StacksDataMapDeleteEventData>
export type NftMintEvent = Record<StacksTransactionEventType.StacksNFTMintEvent, StacksNFTMintEventData>
export type NftTransferEvent = Record<StacksTransactionEventType.StacksNFTTransferEvent, StacksNFTTransferEventData>
export type NftBurnEvent = Record<StacksTransactionEventType.StacksNFTBurnEvent, StacksNFTBurnEventData>
export type FtMintEvent = Record<StacksTransactionEventType.StacksFTMintEvent, StacksFTMintEventData>
export type FtTransferEvent = Record<StacksTransactionEventType.StacksFTTransferEvent, StacksFTTransferEventData>
export type FtBurnEvent = Record<StacksTransactionEventType.StacksFTBurnEvent, StacksFTBurnEventData>

export interface VarValuesData {
  value: string;
  value_type: ClarityAbiType;
  events: Array<[VarSetEvent, number, number]>;
  events_page_size: number;
  events_page_index: number;
}

export interface MapValuesData {
  entries: Array<[[string, string], BlockIdentifier, TransactionIdentifier]>;
  entries_page_size: number;
  entries_page_index: number;
  key_type: ClarityAbiType;
  value_type: ClarityAbiType;
  events: Array<[MapInsertEvent|MapUpdateEvent|MapDeleteEvent, number, number]>;
  events_page_size: number;
  events_page_index: number;
}

export interface NftValuesData {
  tokens: Array<[[string, string], BlockIdentifier, TransactionIdentifier]>;
  tokens_page_size: number;
  tokens_page_index: number;
  token_type: any;
  events: Array<[NftMintEvent|NftTransferEvent|NftBurnEvent, number, number]>;
  events_page_size: number;
  events_page_index: number;
}

export interface FtValuesData {
  balances: Array<[[string, string], BlockIdentifier, TransactionIdentifier]>;
  balances_page_size: number;
  balances_page_index: number;
  events: Array<[FtMintEvent|FtTransferEvent|FtBurnEvent, number, number]>;
  events_page_size: number;
  events_page_index: number;
}

export interface ContractFieldVarUpdate {
  value: string,
  changes: Array<ContractFieldVarChange>,
}

export interface ContractFieldVarChange {
}

export interface ContractFieldMapUpdate {

}

export interface ContractFieldFTUpdate {

}

export interface ContractFieldNFTUpdate {

}

export interface WalletTarget {
  address: string;
}

export interface WalletTargetUpdate {
  address: string;
}

export enum TargetType {
  // Contract = "Contract",
  ContractField = "ContractField",
  // Wallet = "Wallet",
}

export interface StateExplorerWatchState {
  stacks_block_identifier: BlockIdentifier;
  target: WatchedTarget;
}

export interface RequestQueue {
  nextRequest?: Request,
  poll: boolean,
} 

export interface NetworkingState {
  manifestFileWatched?: string;
  bootNetworkStatus?: BootNetworkData;
  protocolIdentifierWatched?: number;
  fieldIdentifierWatched?: [[string, string], BlockIdentifier];
  latestBlockIdentifierKnownByFieldIdentifier: { [fieldIdentifier: string]: BlockIdentifier };
  requestNonce: number,
  nextRequest?: any; // todo: add typing
}

const initialState: NetworkingState = {
  latestBlockIdentifierKnownByFieldIdentifier: {},
  requestNonce: 0,
};

export const networkingSlice = createSlice({
  name: "networking",
  initialState,
  reducers: {
    initiateBootSequence: (
      state: NetworkingState,
      action: PayloadAction<string>
    ) => {
      if (state.manifestFileWatched !== undefined) {
        return;
      }

      if (state.bootNetworkStatus === undefined) {
        state.fieldIdentifierWatched = undefined;
        state.nextRequest = undefined;
        state.protocolIdentifierWatched = undefined;
        state.manifestFileWatched = action.payload;  
      }
    },
    updateBootSequence: (
      state: NetworkingState,
      action: PayloadAction<BootNetworkData>
    ) => {
      if (state.bootNetworkStatus === undefined) {
        state.bootNetworkStatus = action.payload;
      }
      if (action.payload.protocol_deployed === true) {
        state.protocolIdentifierWatched = action.payload.protocol_id;
      }
    },
    updateBlockIdentifierForContractField: (
      state: NetworkingState,
      action: PayloadAction<[string, BlockIdentifier]>
    ) => {
      let [fieldIdentifier, blockIdentifier] = action.payload;
      let knownTip = state.latestBlockIdentifierKnownByFieldIdentifier[fieldIdentifier];
      if (knownTip === undefined) {
        state.latestBlockIdentifierKnownByFieldIdentifier[fieldIdentifier] = blockIdentifier;
        let [key, knownTip] = state.fieldIdentifierWatched!;
        state.fieldIdentifierWatched = [key, blockIdentifier];
      } else {
        if (knownTip.hash !== blockIdentifier.hash) {
          state.latestBlockIdentifierKnownByFieldIdentifier[fieldIdentifier] = blockIdentifier;
          let [key, knownTip] = state.fieldIdentifierWatched!;
          state.fieldIdentifierWatched = [key, blockIdentifier];
        }  
      }
    },
    watchContractField: (
      state: NetworkingState,
      action: PayloadAction<ContractFieldTarget>
    ) => {
      if (state.protocolIdentifierWatched === undefined) {
        return;
      }

      let fieldIdentifier = `${action.payload.contract_identifier}::${action.payload.field_name}`;
      let latestKnownBlock = state.latestBlockIdentifierKnownByFieldIdentifier[fieldIdentifier];
      if (latestKnownBlock === undefined) {
        // Starting with block 1 by default?
        latestKnownBlock = {
          index: 1,
          hash: "",
        };
      }
      state.fieldIdentifierWatched = [[action.payload.contract_identifier, action.payload.field_name], latestKnownBlock];
    },
    buildNextRequest: (
      state: NetworkingState,
      action: PayloadAction<number>
    ) => {
      if (state.manifestFileWatched === undefined) {
        state.nextRequest = undefined;
        return;
      }
  
      if (state.bootNetworkStatus === undefined) {
        state.nextRequest = {
          protocol_id: 1,
          request: {
            "BootNetwork": {
              manifest_path: state.manifestFileWatched
            }
          },
        };
        return;
        // Initiate Call #1
      } else if (state.bootNetworkStatus.protocol_deployed === false) {
        // Initialization still in progress
        state.nextRequest = undefined;
        return;
      } else {
        // Backend is ready, let's continue.
      }
  
      if (state.protocolIdentifierWatched === undefined) {
        state.nextRequest = undefined;
        return;
      }
  
      if (state.fieldIdentifierWatched === undefined) {
        // Nothing being watched. Should just be fetching general blocks informations (todo) 
        state.nextRequest = undefined;
        return;
      }
  
      let [[contractIdentifier, fieldName], latestKnownBlockIdentifier] = state.fieldIdentifierWatched;
  
      let request: StateExplorerWatchState = {
        stacks_block_identifier: latestKnownBlockIdentifier,
        target: {
          "ContractField": {
            contract_identifier: contractIdentifier,
            field_name: fieldName,
          }
        }
      };
  
      state.requestNonce += action.payload;

      state.nextRequest = {
        protocol_id: state.protocolIdentifierWatched,
        nonce: state.requestNonce,
        request: {
          "StateExplorerWatch": request
        },
      };
    }
}});

function isNetworkReady(bootNetworkStatus?: BootNetworkData): boolean {
  return bootNetworkStatus !== undefined && bootNetworkStatus.protocol_deployed;
}

export const {
  watchContractField,
  updateBlockIdentifierForContractField,
  updateBootSequence,
  buildNextRequest,
  initiateBootSequence,
} = networkingSlice.actions;

export const selectNetworkBookStatus = (state: RootState) =>
  state.networking.bootNetworkStatus === undefined ? undefined : state.networking.bootNetworkStatus.contracts.length === 0 ? undefined : state.networking.bootNetworkStatus.status

export const selectManifestFileWatched = (state: RootState) =>
  state.networking.manifestFileWatched

export const selectProtocolName = (state: RootState) => 
 state.networking.bootNetworkStatus === undefined ? "Loading" :  state.networking.bootNetworkStatus.protocol_name

export const selectIsNetworkBooting = (state: RootState) =>
  isNetworkReady(state.networking.bootNetworkStatus) === false

export const selectNextRequest = (state: RootState) =>
  state.networking.nextRequest


// export const selectNextRequest = (state: RootState) => {
//     let nextRequest = undefined;
//     if (state.networking.manifestFileWatched === undefined) {
//       return nextRequest;
//     }

//     if (state.networking.bootNetworkStatus === undefined) {
//       nextRequest = {
//         protocol_id: 1,
//         request: {
//           "BootNetwork": {
//             manifest_path: state.networking.manifestFileWatched
//           }
//         },
//       };
//       // Initiate Call #1
//     } else if (state.networking.bootNetworkStatus.protocol_deployed === false) {
//       // Initialization still in progress
//       return nextRequest;
//     } else {
//       // Backend is ready, let's continue.
//     }

//     if (state.networking.protocolIdentifierWatched === undefined) {
//       return nextRequest;
//     }

//     if (state.networking.fieldIdentifierWatched === undefined) {
//       // Nothing being watched. Should just be fetching general blocks informations (todo) 
//       return nextRequest;
//     }

//     let [[contractIdentifier, fieldName], latestKnownBlockIdentifier] = state.networking.fieldIdentifierWatched;

//     let request: StateExplorerWatchState = {
//       stacks_block_identifier: latestKnownBlockIdentifier,
//       target: {
//         "ContractField": {
//           contract_identifier: contractIdentifier,
//           field_name: fieldName,
//         }
//       }
//     };

//     nextRequest = {
//       protocol_id: state.networking.protocolIdentifierWatched,
//       nonce: state.networking.requestNonce,
//       request: {
//         "StateExplorerWatch": request
//       },
//     };

//     return nextRequest;
// };

export default networkingSlice.reducer;
