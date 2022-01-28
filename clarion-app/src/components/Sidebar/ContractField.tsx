import { Text } from '@primer/react'
import { MouseEvent } from 'react';
import styled from "styled-components"
import { useRootDispatch, useRootSelector } from "../../hooks/useRootSelector";
import { watchContractField } from '../../states/NetworkingState';
import { activateField, selectActiveFieldIdentifier } from "../../states/StateExplorerState";

const Container = styled.div`
    color: rgb(55, 53, 47);
    text-transform: uppercase;
    font-size: 12px;
    font-weight: 600;
    letter-space: 0.03em;
    padding: 4px;
    margin-top: 8px;
    -webkit-user-select: none;      
    -moz-user-select: none;
    -ms-user-select: none;
    user-select: none;
    cursor: default;
    border-radius: 4px;
    background-color: ${(props: { isFieldActive: boolean }) => props.isFieldActive ? "rgba(240, 240, 240, 0.7)" : "clear"};
    &:hover {
        background: rgba(240, 240, 240, 0.7);
}
`

const Tag = styled.div`
    color: rgb(9, 105, 218);
    text-transform: uppercase;
    font-size: 9px;
    font-weight: 600;
    letter-space: 0.03em;
    padding: 4px;
    border-radius: 4px;
    width: 100px;
    height: 40px;
    background-color: rgb(221, 244, 255);
    margin-top: 8px;
    margin-left: 16px;
    -webkit-user-select: none;
    -moz-user-select: none;
    -ms-user-select: none;
    user-select: none;
    cursor: default;
    display: inline;
`

const ContractField = (props: { fieldName: string, fieldType?: string, contractIdentifier: string }) => {
    let dispatch = useRootDispatch();
    const activeFieldIdentifier = useRootSelector(selectActiveFieldIdentifier);
    let fieldIdentifier = `${props.contractIdentifier}::${props.fieldName}`;
    let isFieldActive = activeFieldIdentifier !== undefined && activeFieldIdentifier === fieldIdentifier;

    function handleClick(event: MouseEvent) {
        event.preventDefault();
        let payload = { field_name: props.fieldName, contract_identifier: props.contractIdentifier };
        dispatch(watchContractField(payload));
        dispatch(activateField(payload));
    }
    
    return (
        <Container isFieldActive={isFieldActive} onClick={handleClick}>
            {props.fieldType ? <Tag>{props.fieldType}</Tag> : ''}
            <Text> {props.fieldName}</Text>
        </Container>
    );
};

export { ContractField };