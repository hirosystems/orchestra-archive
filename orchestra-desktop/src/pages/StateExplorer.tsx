import '../App.css';
import { ContractField, Contract, Section } from '../components/Sidebar';
import { Body } from '../components/Main';
import { Box } from '@primer/react'
import { useRootSelector } from "../hooks/useRootSelector";
import { selectFields, selectBookmarks, selectContractsIdentifiers, selectWallets, selectActiveFieldIdentifier } from "../states/StateExplorerState";
import { Wallet } from '../components/Sidebar/Wallet';

function StateExplorer() {

  const contractsIdentifiers = useRootSelector(selectContractsIdentifiers);
  const activeFieldIdentifier = useRootSelector(selectActiveFieldIdentifier);
  const activeBookmarks = useRootSelector(selectBookmarks);

  let bookmarks = [];
  for (let [bookmark, _] of activeBookmarks) {
    let [contractIdentifier, fieldName] = bookmark.split("::");
    bookmarks.push(<ContractField key={0} fieldName={fieldName} contractIdentifier={contractIdentifier} />);
  }

  const wallets = useRootSelector(selectWallets);
  const fields = useRootSelector(selectFields);

  return (
    <div>
      <Box display="flex">
        <Box p={3}>
              <Section name="Bookmarks"/>
                {bookmarks}
              <Section name="Contracts"/>
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
