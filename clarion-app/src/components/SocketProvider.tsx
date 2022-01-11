import { useEffect, useState, createContext, ReactChild } from "react";

const ws = new WebSocket("ws://127.0.0.1:2404");

const SocketContext = createContext(ws);

interface ISocketProvider {
  children: ReactChild;
}

const SocketProvider = (props: ISocketProvider) => (
  <SocketContext.Provider value={ws}>{props.children}</SocketContext.Provider>
);

export { SocketContext, SocketProvider };