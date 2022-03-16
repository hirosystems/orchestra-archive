import { TypedUseSelectorHook, useDispatch, useSelector } from "react-redux";
import type { RootState, RootDispatch } from "../stores/root";

// Use throughout your app instead of plain `useDispatch` and `useSelector`
export const useRootDispatch = () => useDispatch<RootDispatch>();
export const useRootSelector: TypedUseSelectorHook<RootState> = useSelector;
