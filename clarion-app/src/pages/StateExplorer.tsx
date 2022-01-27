import '../App.css';
import { ContractField, Contract, Section } from '../components/Sidebar';
import { Body } from '../components/Main';

import { Box } from '@primer/react'
import { Heading, Text } from '@primer/react'
import { BlockHeader } from '../components/BlockHeader';
import { useRootSelector, useRootDispatch } from "../hooks/useRootSelector";
import { selectFields, selectContractsIdentifiers, selectWallets, selectActiveFieldIdentifier } from "../states/StateExplorerState";
import { initializeStateExplorer } from '../states/NetworkingState';
import { Wallet } from '../components/Sidebar/Wallet';

function StateExplorer() {

  let dispatch = useRootDispatch();
  let hardcodedProjectPath = "/Users/ludovic/Coding/clarinet/clarinet-cli/examples/counter/Clarinet.toml";
  dispatch(initializeStateExplorer(hardcodedProjectPath));

  const contractsIdentifiers = useRootSelector(selectContractsIdentifiers);
  const activeFieldIdentifier = useRootSelector(selectActiveFieldIdentifier);
  const wallets = useRootSelector(selectWallets);
  const fields = useRootSelector(selectFields);

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
                return <Contract key={i} contractIdentifier={contractIdentifier}/>
              })}
            <Section name="Wallets"/>
            {wallets.map((wallet, i) => {
              let fields = [];
              fields.push(
                <Wallet key={i} walletAddress={wallet}/>
              )
              return fields
            }
            )}
        </Box>
        <Box flexGrow={1} p={3}>
          <Body field={activeFieldIdentifier && fields[activeFieldIdentifier] ? fields[activeFieldIdentifier] : undefined }/>
        </Box>
      </Box>
    </div>
  );
}

export default StateExplorer;
