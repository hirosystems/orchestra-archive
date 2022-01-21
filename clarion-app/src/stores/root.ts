import { configureStore } from '@reduxjs/toolkit';
import stateExplorerReducer from '../states/StateExplorerState';
import networkingReducer from '../states/NetworkingState';

export const rootStore = configureStore({
  reducer: {
    stateExplorer: stateExplorerReducer,
    networking: networkingReducer,
  },
});

export type RootDispatch = typeof rootStore.dispatch;
export type RootState = ReturnType<typeof rootStore.getState>;
