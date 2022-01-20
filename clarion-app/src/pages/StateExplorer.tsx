import '../App.css';
import { ContractFieldLink } from '../components/ContractFieldLink';
import { Box } from '@primer/react'
import { SideNav, Text } from '@primer/react'
import { useCallback, useEffect, useState } from "react";
import { useSocket } from "../hooks/useSocket";
import { useInterval } from "../hooks/useInterval";
import { useMap } from "../hooks/useMap";
import { BlockHeader } from '../components/BlockHeader';
import { PollState, ContractStateReady, PollStateUpdate, PollStateInitialization } from '../types';
import { StacksContractInterface } from "../types";

function StateExplorer() {
  const [localState, setLocalState] = useState({
    key: PollState.None,
    value: {}
  });
  const [isPolling, setPolling] = useState<boolean>(false)
  const [expectedContracts, setExpectedContracts] = useState<string[]>([])

  const socket = useSocket();
  const [block, setBlock] = useState("-");
  const [contractStore, contractStoreActions] = useMap<string, StacksContractInterface>();

  useInterval(
    () => {
      let request = Object.fromEntries([localState].map(e => [e.key, e.value]))
      let payload = {
        project_id: 0,
        request: request,
      };
      socket.send(JSON.stringify(payload));
  }, isPolling ? 5000 : null);

  const onMessage = useCallback((message) => {
    const data: PollStateUpdate = JSON.parse(message?.data);
    if (data.update.Initialization) {
      let value: any = {...data.update.Initialization};
      let contracts: Array<ContractStateReady> = value.contracts;
      let contractsIds = [];
      for (const contract of contracts) {
        contractStoreActions.set(contract.contract_identifier, contract.interface);
        contractsIds.push(contract.contract_identifier);
      }
      setExpectedContracts([...contractsIds]);
    }

  }, []);

  // const onLocalStateUpdate = useCallback((state: PollStateData) => {
  //   console.log(state);
  // }, [setLocalState]);

  useEffect(() => {
    socket.addEventListener("message", onMessage);
    let manifestPath = "/Users/ludovic/Coding/clarinet/clarinet-cli/examples/counter/Clarinet.toml";
    let payload: PollStateInitialization = {
      manifest_path: manifestPath,
    }

    setLocalState({
      key: PollState.Initialization,
      value: payload
    });

    setPolling(true);

    return () => {
      socket.removeEventListener("message", onMessage);
    };
  }, [socket, onMessage]);

  return (
    <div>
      {/* <BlockHeader block={block} /> */}
      <Box display="flex">
        <Box p={3}>
          <SideNav bordered sx={{ width: 280 }}>
            <SideNav.Link href="#url">
              <Text>Contracts</Text>

              {/* <FilteredSearch>
                <Dropdown>
                <Dropdown.Button>Field Type</Dropdown.Button>
                <Dropdown.Menu direction="se">
                  <Dropdown.Item>VAR</Dropdown.Item>
                  <Dropdown.Item>MAP</Dropdown.Item>
                  <Dropdown.Item>FT</Dropdown.Item>
                  <Dropdown.Item>NFT</Dropdown.Item>
                </Dropdown.Menu>
              </Dropdown>
                <TextInput sx={{ pl: 1 }} icon={SearchIcon} />
              </FilteredSearch> */}
            </SideNav.Link>
            {expectedContracts.map((contract, i) => {

              let fields = [];
              fields.push(
                <SideNav.Link href="#url">
                  <Text>{contract.split('.')[1]}</Text>
                </SideNav.Link>
              )
              for (const v of contractStore.get(contract)!.variables) {
                fields.push(
                  <ContractFieldLink fieldName={v.name} fieldType="var" contractIdentifier={contract} />
                )
              }
              for (const v of contractStore.get(contract)!.maps) {
                fields.push(
                  <ContractFieldLink fieldName={v.name} fieldType="map" contractIdentifier={contract} />
                )
              }
              for (const v of contractStore.get(contract)!.fungible_tokens) {
                fields.push(
                  <ContractFieldLink fieldName={v.name} fieldType="ft" contractIdentifier={contract} />
                )
              }
              for (const v of contractStore.get(contract)!.non_fungible_tokens) {
                fields.push(
                  <ContractFieldLink fieldName={v.name} fieldType="nft" contractIdentifier={contract} />
                )
              }
              return fields
            }
            )}          
          </SideNav>
        </Box>
        <Box flexGrow={1} p={3}>
        <div className="blankslate blankslate-large">
  <img src="https://ghicons.github.com/assets/images/blue/png/Pull%20request.png" alt="" className="mb-3" />
  <h3 className="mb-1">You don’t seem to have any pull requests.</h3>
  <p>Pull requests help you discuss potential changes before they are merged into the base branch.</p>
  <button className="btn btn-primary my-3" type="button">New pull request</button>
  <p><button className="btn-link" type="button">Learn more</button></p>
</div>
        </Box>
      </Box>
    </div>
  );
}

export default StateExplorer;
