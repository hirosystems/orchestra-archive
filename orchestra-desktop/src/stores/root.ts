import { configureStore } from "@reduxjs/toolkit";
import stateExplorerReducer from "../states/StateExplorerState";
import networkingReducer from "../states/NetworkingState";
import blocksExplorerReducer from "../states/BlocksExplorerState";

export const rootStore = configureStore({
  reducer: {
    stateExplorer: stateExplorerReducer,
    blocksExplorer: blocksExplorerReducer,
    networking: networkingReducer,
  },
});

export type RootDispatch = typeof rootStore.dispatch;
export type RootState = ReturnType<typeof rootStore.getState>;
