import { createSlice, PayloadAction } from "@reduxjs/toolkit";
import { RootState } from "../stores/root";
import { ClarityAbiType, Contract } from "../types";
import { BitcoinBlockMetadata, Block, BlockIdentifier, StacksDataMapDeleteEventData, StacksDataMapInsertEventData, StacksDataMapUpdateEventData, StacksDataVarSetEventData, StacksNFTBurnEventData, StacksNFTMintEventData, StacksNFTTransferEventData, StacksFTBurnEventData, StacksFTMintEventData, StacksFTTransferEventData, StacksTransactionEventType, TransactionIdentifier } from "../types/clarinet";

export enum ActiveFeature {
  StateExplorer,
}

export type StateExplorerStateUpdateWatch = Record<"StateExplorerWatch", StateExplorerStateUpdateWatchData>
export type StateExplorerStateUpdateInit = Record<"StateExplorerInitialization", StateExplorerStateUpdateInitData>

export interface StateExplorerStateUpdate {
  update: StateExplorerStateUpdateWatch | StateExplorerStateUpdateInit;
}

export interface StateExplorerStateUpdateInitData {
  contracts: Array<Contract>;
}

export type VarValues = Record<"Var", VarValuesData>
export type MapValues = Record<"Map", MapValuesData>
export type NftValues = Record<"Nft", NftValuesData>
export type FtValues = Record<"Ft", FtValuesData>

export interface StateExplorerStateUpdateWatchData {
  stacks_chain_blocks: Array<Block>;
  bitcoin_chain_blocks: Array<Block>
  contract_identifier: string;
  field_name: string;
  field_values: VarValues | MapValues | NftValues | FtValues;
}

export enum StateExplorerState {
  None = "None",
  Initialization = "StateExplorerInitialization",
  Sleep = "StateExplorerSleep",
  Watch = "StateExplorerWatch",
}

export interface Request {
  protocol_id: number;
  request: any;
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
  events: Array<VarSetEvent>;
  events_page_size: number;
  events_page_index: number;
}

export interface MapValuesData {
  entries: Array<[[string, string], BlockIdentifier, TransactionIdentifier]>;
  entries_page_size: number;
  entries_page_index: number;
  key_type: ClarityAbiType;
  value_type: ClarityAbiType;
  events: Array<MapInsertEvent|MapUpdateEvent|MapDeleteEvent>;
  events_page_size: number;
  events_page_index: number;
}

export interface NftValuesData {
  tokens: Array<[[string, string], BlockIdentifier, TransactionIdentifier]>;
  tokens_page_size: number;
  tokens_page_index: number;
  token_type: any;
  events: Array<NftMintEvent|NftTransferEvent|NftBurnEvent>;
  events_page_size: number;
  events_page_index: number;
}

export interface FtValuesData {
  balances: Array<[[string, string], BlockIdentifier, TransactionIdentifier]>;
  balances_page_size: number;
  balances_page_index: number;
  events: Array<FtMintEvent|FtTransferEvent|FtBurnEvent>;
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
  bitcoin_block_identifier: BlockIdentifier;
  target: WatchedTarget;
}

export interface StateExplorerNetworkingState {
  active: boolean;
  state: StateExplorerState;
  manifestPath?: string;
  broadcastableState?:
    | StateExplorerInitializationState
    | StateExplorerWatchState
    | StateExplorerPauseState;
}

export interface RequestQueue {
  nextRequest?: Request,
  poll: boolean,
} 

export interface NetworkingState {
  activeFeature?: ActiveFeature;
  requestQueue: RequestQueue;
  stateExplorer: StateExplorerNetworkingState;
}

const initialState: NetworkingState = {
  stateExplorer: {
    active: false,
    state: StateExplorerState.None,
  },
  requestQueue: {
    nextRequest: undefined,
    poll: false,
  }
};

export const networkingSlice = createSlice({
  name: "networking",
  initialState,
  reducers: {
    initializeStateExplorer: (
      state: NetworkingState,
      action: PayloadAction<string>
    ) => {
      // Guard duplicate messages
      if (state.stateExplorer.active) {
        return;
      }

      state.stateExplorer.active = true;
      if (
        state.stateExplorer.state === StateExplorerState.None ||
        action.payload !== state.stateExplorer.manifestPath
      ) {
        state.stateExplorer.state = StateExplorerState.Initialization;
        state.stateExplorer.manifestPath = action.payload;
      }
      state.stateExplorer.broadcastableState = {
        manifest_path: action.payload,
      };

      let request = Object.fromEntries([
        [
          StateExplorerState.Initialization,
          state.stateExplorer.broadcastableState,
        ],
      ]);
      let payload = {
        protocol_id: 1,
        request: request,
      };

      state.requestQueue = {
        nextRequest: payload,
        poll: false,
      };
    },
    // watchContract: (
    //   state: NetworkingState,
    //   action: PayloadAction<Contract>
    // ) => {
    //   state.stateExplorer.active = true;

    //   let target = Object.fromEntries([[TargetType.Contract, action.payload]]);

    //   let request = Object.fromEntries([[StateExplorerState.Watch, target]]);
    //   let payload = {
    //     protocol_id: 0,
    //     request: request,
    //   };

    //   state.request = payload;
    // },
    dequeueRequest: (
      state: NetworkingState,
      action: PayloadAction<Request>
    ) => {
      if (action.payload === state.requestQueue.nextRequest && !state.requestQueue.poll) {
        state.requestQueue.nextRequest = undefined;
      }
    },
    watchContractField: (
      state: NetworkingState,
      action: PayloadAction<ContractFieldTarget>
    ) => {
      state.stateExplorer.active = true;

      let inner: StateExplorerWatchState = {
        stacks_block_identifier: {
          index: 1,
          hash: "1",
        },
        bitcoin_block_identifier: {
          index: 1,
          hash: "1",
        },
        target: {
          "ContractField": {
            contract_identifier: action.payload.contract_identifier,
            field_name: action.payload.field_name,
          }
        }
      };
  
      // let target = Object.fromEntries([
      //   [TargetType.ContractField, action.payload],
      // ]);

      // let request = Object.fromEntries([[StateExplorerState.Watch, target]]);
      let payload = {
        protocol_id: 1,
        request: {
          "StateExplorerWatch": inner
        },
      };

      state.requestQueue = {
        nextRequest: payload,
        poll: true,
      };
    },
    watchWallet: (state: NetworkingState, action: PayloadAction<WalletTarget>) => {
      state.stateExplorer.active = true;

      // let target = Object.fromEntries([[TargetType.Wallet, action.payload]]);

      // let request = Object.fromEntries([[StateExplorerState.Watch, target]]);
      // let payload = {
      //   protocol_id: 0,
      //   request: request,
      // };

      // state.requestQueue = {
      //   nextRequest: payload,
      //   poll: true,
      // };
    },
  },
});

export const {
  initializeStateExplorer,
  watchContractField,
  watchWallet,
  dequeueRequest,
} = networkingSlice.actions;

export const selectActiveFeature = (state: RootState) =>
  state.networking.activeFeature;

export const selectStateExplorerNetworkingState = (state: RootState) =>
  state.networking.stateExplorer;

export const selectStateExplorerBroadcastableState = (state: RootState) =>
  state.networking.stateExplorer.broadcastableState;

export const selectRequestQueue = (state: RootState) => state.networking.requestQueue;

export default networkingSlice.reducer;
