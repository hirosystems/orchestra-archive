import '../App.css';
import { ContractField, Contract, Section, Title } from '../components/Sidebar';
import { Box } from '@primer/react'
import { Heading, Text } from '@primer/react'
import { BlockHeader } from '../components/BlockHeader';
import { useRootSelector, useRootDispatch } from "../hooks/useRootSelector";
import { selectContracts, selectContractsIdentifiers, selectActiveContractIdentifier } from "../states/StateExplorerState";
import { initializeStateExplorer } from '../states/NetworkingState';

function StateExplorer() {

  let dispatch = useRootDispatch();
  let hardcodedProjectPath = "/Users/ludovic/Coding/clarinet/clarinet-cli/examples/counter/Clarinet.toml";
  dispatch(initializeStateExplorer(hardcodedProjectPath));

  const contracts = useRootSelector(selectContracts);
  const contractsIdentifiers = useRootSelector(selectContractsIdentifiers);

  return (
    <div>
      {/* <Text content="State Explorer" size="large" weight="bold" /> */}
      {/* <BlockHeader block={block} /> */}
      <Box display="flex">
        <Box p={3}>
              <Section name="Contracts"/>
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
            {contractsIdentifiers.map((contractIdentifier, i) => {
              let fields = [];

              fields.push(
                <Contract contractIdentifier={contractIdentifier}/>
              )
              let index = 0;
              for (const v of contracts.get(contractIdentifier)!.variables) {
                fields.push(
                  <ContractField key={index} fieldName={v.name} fieldType="var" contractIdentifier={contractIdentifier} />
                )
                index += 1;
              }
              for (const v of contracts.get(contractIdentifier)!.maps) {
                fields.push(
                  <ContractField key={index} fieldName={v.name} fieldType="map" contractIdentifier={contractIdentifier} />
                )
                index += 1;
              }
              for (const v of contracts.get(contractIdentifier)!.fungible_tokens) {
                fields.push(
                  <ContractField key={index} fieldName={v.name} fieldType="ft" contractIdentifier={contractIdentifier} />
                )
                index += 1;
              }
              for (const v of contracts.get(contractIdentifier)!.non_fungible_tokens) {
                fields.push(
                  <ContractField key={index} fieldName={v.name} fieldType="nft" contractIdentifier={contractIdentifier} />
                )
                index += 1;
              }
              return fields
            }
            )}    
            <Section name="Wallets"/>
        </Box>
        <Box flexGrow={1} p={3}>

        </Box>
      </Box>
    </div>
  );
}

export default StateExplorer;
