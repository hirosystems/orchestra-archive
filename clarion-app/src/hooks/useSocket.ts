import { SocketContext } from "../components/SocketProvider";
import { useContext } from "react";

const useSocket = () => {
  const socket = useContext(SocketContext);

  return socket;
};

export { useSocket };