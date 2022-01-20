import { BranchName, SideNav, Text } from '@primer/react'
import { MouseEvent } from 'react';

const ContractFieldLink = (props: { fieldName: String, fieldType: String, contractIdentifier: String }) => {

    function handleClick(event: MouseEvent) {
        event.preventDefault();
        alert(event.currentTarget.tagName); 
    }
    
    return (
        <SideNav.Link href="#url" onClick={handleClick}>
            <BranchName>{props.fieldType}</BranchName>
            <Text> {props.fieldName}</Text>
        </SideNav.Link>
    );
};

export { ContractFieldLink };