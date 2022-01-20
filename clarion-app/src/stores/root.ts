import { configureStore } from '@reduxjs/toolkit';
import stateExplorerReducer from '../states/StateExplorerState';

export const rootStore = configureStore({
  reducer: {
    stateExplorer: stateExplorerReducer,
  },
});

export type RootDispatch = typeof rootStore.dispatch;
export type RootState = ReturnType<typeof rootStore.getState>;
