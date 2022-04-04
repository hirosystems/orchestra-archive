import '../App.css';
import { ContractField, Contract, Section } from '../components/Sidebar';
import { Body } from '../components/Main';
import { useRootSelector } from "../hooks/useRootSelector";
import { selectFields, selectBookmarks, selectOrderedStatefulContractsIdentifiers, selectWallets, selectActiveFieldIdentifier } from "../states/StateExplorerState";
import { Wallet } from '../components/Sidebar/Wallet';
import styled from "styled-components";

export const Container = styled.div`
`

export const LeftPanel = styled.div`
position: absolute;
left: 72px;
width: 270px;
top: 32px;
bottom 0;
padding-left: 8px;
height: 800px;
overflow-x: hidden;
overflow-y: auto; 
`

export const Navigation = styled.div`
background-color: #121212;
height: 800px;
`

export const MainPanel = styled.div`
padding-top: 30px;
padding-left: 300px;
background-color: #121212;
padding-bottom: 200px;
`

function StateExplorer() {

  const contractsIdentifiers = useRootSelector(selectOrderedStatefulContractsIdentifiers);
  const activeFieldIdentifier = useRootSelector(selectActiveFieldIdentifier);
  const activeBookmarks = useRootSelector(selectBookmarks);

  let bookmarks = [];
  if (activeBookmarks.length > 0) {
    bookmarks.push(<Section name="Bookmarks" />);
  }
  for (let [bookmark, _] of activeBookmarks) {
    let [contractIdentifier, fieldName] = bookmark.split("::");
    bookmarks.push(<ContractField key={0} fieldName={fieldName} contractIdentifier={contractIdentifier} />);
  }

  const wallets = useRootSelector(selectWallets);
  const fields = useRootSelector(selectFields);

  return (
    <Container>
      <LeftPanel>
        <Navigation>
          {bookmarks}
          <Section name="Stateful Contracts" />
          {contractsIdentifiers.map((contractIdentifier, i) => {
            return <Contract key={i} contractIdentifier={contractIdentifier} />
          })}
          <Section name="Wallets" />
          {wallets.map((wallet, i) => {
            let fields = [];
            fields.push(
              <Wallet key={i} walletAddress={wallet} />
            )
            return fields
          }
          )}
        </Navigation>
      </LeftPanel>
      <MainPanel>
        <Body field={activeFieldIdentifier && fields[activeFieldIdentifier] ? fields[activeFieldIdentifier] : undefined} />
      </MainPanel>
    </Container>
  );
}

export default StateExplorer;
