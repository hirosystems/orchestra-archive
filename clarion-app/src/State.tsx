import React from 'react';
import './App.css';
import {Box} from '@primer/react'
import {BranchName, SideNav, Text, FilteredSearch, CounterLabel, Dropdown, TextInput} from '@primer/react'
import {SearchIcon} from '@primer/octicons-react'
import { useCallback, useEffect, useState } from "react";
import { useSocket } from "./hooks/useSocket";
import {BlockHeader} from './components/BlockHeader';
import { BlockIdentifier } from "../../../clarinet/clarinet-cli/node-bindings/lib/types";

function Data() {
  const socket = useSocket();
  const [block, setBlock] = useState("-");

  const onMessage = useCallback((message) => {
    const data = JSON.parse(message?.data);
    setBlock(data.msg);
  }, []);

  useEffect(() => {
    socket.addEventListener("message", onMessage);

    let block_identifier: BlockIdentifier = {
      index: 0,
      hash: "0x00000000000000000000000000000000"
    };

    function pollState() {
        let payload = {
          current_block_identifier: {
            index: 0,
            hash: "0x00000000000000000000000000000000"
          },
          project_id: 0,
          request: {
            "StateExplorer" : {
              contract_identifier: "",
              field: "",
            },
          },
        };
        socket.send(JSON.stringify(payload));
        setTimeout(pollState, 3000)
    }
  
    pollState();

    return () => {
      socket.removeEventListener("message", onMessage);
    };
  }, [socket, onMessage]);

  return (
      <div>
        <BlockHeader block={block}/>
        <Box display="flex">
        <Box p={3}>
          <SideNav bordered sx={{ maxWidth: 280 }}>
            <SideNav.Link href="#url">
            <FilteredSearch>
              {/* <Dropdown>
                <Dropdown.Button>Field Type</Dropdown.Button>
                <Dropdown.Menu direction="se">
                  <Dropdown.Item>VAR</Dropdown.Item>
                  <Dropdown.Item>MAP</Dropdown.Item>
                  <Dropdown.Item>FT</Dropdown.Item>
                  <Dropdown.Item>NFT</Dropdown.Item>
                </Dropdown.Menu>
              </Dropdown> */}
              <TextInput sx={{ pl: 1 }} icon={SearchIcon} />
            </FilteredSearch>
            </SideNav.Link>
            <SideNav.Link href="#url" selected>
              <Text>counter.clar</Text>
            </SideNav.Link>
            <SideNav bordered variant="lightweight" sx={{ py: 2, pl: 2, backgroundColor: "sidenav.selectedBg" }}>
              <SideNav.Link href="#url" selected>
                <BranchName>var</BranchName>
                <Text> counter-id</Text>
              </SideNav.Link>
              <SideNav.Link href="#url">
                <BranchName>map</BranchName>
                <Text> counter-values</Text>
                <CounterLabel scheme="secondary">13</CounterLabel>
              </SideNav.Link>
              <SideNav.Link href="#url">
                <BranchName>nft</BranchName>
                <Text> counter-nft</Text>
              </SideNav.Link>
            </SideNav>
          </SideNav>
          </Box>
        <Box flexGrow={1} p={3}>
          <Box color="fg.muted" bg="canvas.subtle" p={3}>
            Hello
          </Box>
          <Box borderColor="white" borderWidth={10} borderStyle="solid"></Box>
          <Box color="fg.muted" bg="canvas.subtle" p={3}>
            Hello
          </Box>
        </Box>
      </Box>
    </div>
  );
}

export default Data;
