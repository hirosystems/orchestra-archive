import { createSlice, PayloadAction } from "@reduxjs/toolkit";
import { Contract, StacksContractInterface } from "../types";
import { RootState } from "../stores/root";
import {
  ContractFieldTarget,
  StateExplorerStateUpdateWatchData,
} from "./NetworkingState";
import { BlockIdentifier } from "../types/clarinet";

export interface StateExplorerState {
  initialized: boolean;
  contractsIdentifiers: Array<string>;
  bookmarks: { [fieldIdentifier: string]: boolean };
  notifications: { [fieldIdentifier: string]: boolean };
  contracts: { [contractIdentifier: string]: StacksContractInterface };
  fields: { [fieldIdentifier: string]: StateExplorerStateUpdateWatchData };
  latestFieldsBlockIdentifiers: { [fieldIdentifier: string]: BlockIdentifier };
  wallets: Array<string>;
  activeContractIdentifier?: string;
  activeFieldIdentifier?: string;
}

const initialState: StateExplorerState = {
  initialized: false,
  contractsIdentifiers: [],
  wallets: [],
  notifications: {},
  bookmarks: {},
  contracts: {},
  fields: {},
  latestFieldsBlockIdentifiers: {},
  activeContractIdentifier: undefined,
  activeFieldIdentifier: undefined,
};

export const stateExplorerSlice = createSlice({
  name: "stateExplorer",
  initialState,
  reducers: {
    activateDefaultField: (state: StateExplorerState) => {
      for (let contractIdentifier of state.contractsIdentifiers) {
        let contract = state.contracts[contractIdentifier];
        if (contract.variables.length > 0) {
          state.activeFieldIdentifier = `${contractIdentifier}::${contract.variables[0].name}`;
          state.activeContractIdentifier = contractIdentifier;
          break;
        } else if (contract.maps.length > 0) {
          state.activeFieldIdentifier = `${contractIdentifier}::${contract.maps[0].name}`;
          state.activeContractIdentifier = contractIdentifier;
          break;
        } else if (contract.fungible_tokens.length > 0) {
          state.activeFieldIdentifier = `${contractIdentifier}::${contract.fungible_tokens[0].name}`;
          state.activeContractIdentifier = contractIdentifier;
          break;
        } else if (contract.non_fungible_tokens.length > 0) {
          state.activeFieldIdentifier = `${contractIdentifier}::${contract.non_fungible_tokens[0].name}`;
          state.activeContractIdentifier = contractIdentifier;
          break;
        }
      }
    },
    activateField: (
      state: StateExplorerState,
      action: PayloadAction<ContractFieldTarget>
    ) => {
      state.activeFieldIdentifier = `${action.payload.contract_identifier}::${action.payload.field_name}`;
      state.activeContractIdentifier = action.payload.contract_identifier;
    },
    updateContracts: (
      state: StateExplorerState,
      action: PayloadAction<Array<Contract>>
    ) => {
      state.contractsIdentifiers = [];
      state.contracts = {};
      for (const contract of action.payload) {
        state.contracts[contract.contract_identifier] = contract.interface;
        state.contractsIdentifiers.push(contract.contract_identifier);
      }
    },
    updateField: (
      state: StateExplorerState,
      action: PayloadAction<StateExplorerStateUpdateWatchData>
    ) => {
      let fieldId = "";
      let lastBlockId = undefined;
      if ("Var" in action.payload.field_values) {
        fieldId = `${action.payload.contract_identifier}::${action.payload.field_name}`;
        if (action.payload.stacks_blocks.length > 0) {
          lastBlockId =
            action.payload.stacks_blocks[
              action.payload.stacks_blocks.length - 1
            ].block_identifier;
        }
      } else if ("Map" in action.payload.field_values) {
        fieldId = `${action.payload.contract_identifier}::${action.payload.field_name}`;
        if (action.payload.stacks_blocks.length > 0) {
          lastBlockId =
            action.payload.stacks_blocks[
              action.payload.stacks_blocks.length - 1
            ].block_identifier;
        }
      } else if ("Ft" in action.payload.field_values) {
        fieldId = `${action.payload.contract_identifier}::${action.payload.field_name}`;
        if (action.payload.stacks_blocks.length > 0) {
          lastBlockId =
            action.payload.stacks_blocks[
              action.payload.stacks_blocks.length - 1
            ].block_identifier;
        }
      } else if ("Nft" in action.payload.field_values) {
        fieldId = `${action.payload.contract_identifier}::${action.payload.field_name}`;
        if (action.payload.stacks_blocks.length > 0) {
          lastBlockId =
            action.payload.stacks_blocks[
              action.payload.stacks_blocks.length - 1
            ].block_identifier;
        }
      }
      state.fields[`${fieldId}`] = action.payload;
      if (lastBlockId !== undefined) {
        state.latestFieldsBlockIdentifiers[`${fieldId}`] = lastBlockId;
      }
    },
    toggleBookmark: (
      state: StateExplorerState,
      action: PayloadAction<string>
    ) => {
      state.bookmarks[action.payload] = isEnabled(
        state.bookmarks,
        action.payload
      )
        ? false
        : true;
    },
    toggleNotification: (
      state: StateExplorerState,
      action: PayloadAction<string>
    ) => {
      state.notifications[action.payload] = !isEnabled(
        state.notifications,
        action.payload
      );
    },
  },
});

function isEnabled(
  map: { [fieldIdentifier: string]: boolean },
  fieldIdentifier?: string
): boolean {
  return (
    fieldIdentifier !== undefined &&
    map[fieldIdentifier] !== undefined &&
    map[fieldIdentifier] === true
  );
}

export const {
  activateDefaultField,
  activateField,
  updateContracts,
  updateField,
  toggleBookmark,
  toggleNotification,
} = stateExplorerSlice.actions;

export const selectContracts = (state: RootState) =>
  state.stateExplorer.contracts;

export const selectContractsIdentifiers = (state: RootState) =>
  state.stateExplorer.contractsIdentifiers;

export const selectBookmarks = (state: RootState) =>
  Object.entries(state.stateExplorer.bookmarks).filter(([k, v]) => v === true);

export const selectWallets = (state: RootState) => state.stateExplorer.wallets;

export const selectFields = (state: RootState) => state.stateExplorer.fields;

export const selectActiveContractIdentifier = (state: RootState) =>
  state.stateExplorer.activeContractIdentifier;

export const selectActiveFieldIdentifier = (state: RootState) =>
  state.stateExplorer.activeFieldIdentifier;

export const selectLatestKnownBlockIdentifier = (state: RootState) =>
  state.stateExplorer.activeFieldIdentifier &&
  state.stateExplorer.latestFieldsBlockIdentifiers[
    state.stateExplorer.activeFieldIdentifier
  ];

export const isNotificationEnabled = (state: RootState) =>
  isEnabled(
    state.stateExplorer.notifications,
    state.stateExplorer.activeFieldIdentifier
  );

export const isBookmarkEnabled = (state: RootState) =>
  isEnabled(
    state.stateExplorer.bookmarks,
    state.stateExplorer.activeFieldIdentifier
  );

export default stateExplorerSlice.reducer;
