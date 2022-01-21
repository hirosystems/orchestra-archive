import '../App.css';
import { ContractFieldLink } from '../components/ContractFieldLink';
import { Box } from '@primer/react'
import { SideNav, Text } from '@primer/react'
import { BlockHeader } from '../components/BlockHeader';
import { useRootSelector, useRootDispatch } from "../hooks/useRootSelector";
import { selectContracts, selectContractsIdentifiers } from "../states/StateExplorerState";
import { initializeStateExplorer } from '../states/NetworkingState';

function StateExplorer() {

  let dispatch = useRootDispatch();
  let hardcodedProjectPath = "/Users/ludovic/Coding/clarinet/clarinet-cli/examples/counter/Clarinet.toml";
  dispatch(initializeStateExplorer(hardcodedProjectPath));

  const contracts = useRootSelector(selectContracts);
  const contractsIdentifiers = useRootSelector(selectContractsIdentifiers);
  
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
            {contractsIdentifiers.map((contractIdentifier, i) => {
              let fields = [];
              fields.push(
                <SideNav.Link href="#url">
                  <Text>{contractIdentifier.split('.')[1]}</Text>
                </SideNav.Link>
              )
              let index = 0;
              for (const v of contracts.get(contractIdentifier)!.variables) {
                fields.push(
                  <ContractFieldLink key={index} fieldName={v.name} fieldType="var" contractIdentifier={contractIdentifier} />
                )
                index += 1;
              }
              for (const v of contracts.get(contractIdentifier)!.maps) {
                fields.push(
                  <ContractFieldLink key={index} fieldName={v.name} fieldType="map" contractIdentifier={contractIdentifier} />
                )
                index += 1;
              }
              for (const v of contracts.get(contractIdentifier)!.fungible_tokens) {
                fields.push(
                  <ContractFieldLink key={index} fieldName={v.name} fieldType="ft" contractIdentifier={contractIdentifier} />
                )
                index += 1;
              }
              for (const v of contracts.get(contractIdentifier)!.non_fungible_tokens) {
                fields.push(
                  <ContractFieldLink key={index} fieldName={v.name} fieldType="nft" contractIdentifier={contractIdentifier} />
                )
                index += 1;
              }
              return fields
            }
            )}          
          </SideNav>
        </Box>
        <Box flexGrow={1} p={3}>
        </Box>
      </Box>
    </div>
  );
}

export default StateExplorer;
