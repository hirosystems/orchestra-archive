import { createSlice, PayloadAction } from "@reduxjs/toolkit";
import { RootState } from "../stores/root";
import { BlockIdentifier } from "../types/clarinet";

export enum ActiveFeature {
  StateExplorer,
}

export enum StateExplorerState {
  None = "None",
  Initialization = "StateExplorerInitialization",
  Sync = "StateExplorerSync",
  Active = "StateExplorerActive",
}

export interface Request {
  project_id: number;
  request: any;
}

export interface StateExplorerInitializationState {
  manifest_path: string;
}

export interface StateExplorerSyncState {
  stacks_chain_tip?: BlockIdentifier;
  bitcoin_chain_tip?: BlockIdentifier;
  expected_contracts_identifiers: string[];
}

export interface StateExplorerActiveState {
  stacks_chain_tip: BlockIdentifier;
  bitcoin_chain_tip: BlockIdentifier;
  contract_identifier: string;
  field: string;
}

export interface StateExplorerNetworkingState {
  active: boolean;
  state: StateExplorerState;
  manifestPath?: string;
  broadcastableState?:
    | StateExplorerInitializationState
    | StateExplorerSyncState
    | StateExplorerActiveState;
}

export interface NetworkingState {
  activeFeature?: ActiveFeature;
  request?: Request;
  stateExplorer: StateExplorerNetworkingState;
}

const initialState: NetworkingState = {
  stateExplorer: {
    active: false,
    state: StateExplorerState.None,
  },
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
        project_id: 0,
        request: request,
      };

      state.request = payload;
    },
    syncStateExplorer: (
      state: NetworkingState,
      action: PayloadAction<Array<string>>
    ) => {
      state.stateExplorer.active = true;
    },
  },
});

export const { initializeStateExplorer } = networkingSlice.actions;

export const selectActiveFeature = (state: RootState) =>
  state.networking.activeFeature;

export const selectStateExplorerNetworkingState = (state: RootState) =>
  state.networking.stateExplorer;

export const selectStateExplorerBroadcastableState = (state: RootState) =>
  state.networking.stateExplorer.broadcastableState;

export const selectRequest = (state: RootState) => state.networking.request;

export const selectShouldPoll = (state: RootState) =>
  state.networking.stateExplorer.active;

export default networkingSlice.reducer;
