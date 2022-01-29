import { MouseEvent } from 'react';
import styled from "styled-components"
import { useRootDispatch, useRootSelector } from "../../hooks/useRootSelector";
import { watchContractField } from '../../states/NetworkingState';
import { activateField, selectActiveFieldIdentifier } from "../../states/StateExplorerState";

const Container = styled.div`
display: flex;
justify-content: flex-end;
    color: rgb(55, 53, 47);
    text-transform: uppercase;
    font-size: 12px;
    font-weight: 600;
    letter-space: 0.03em;
    padding: 4px;
    padding-left: 0px;
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
    color: ${(props: { backgroundColor: string, color: string }) => props.color};;
    text-transform: uppercase;
    font-size: 9px;
    font-weight: 600;
    letter-space: 0.03em;
    padding: 4px;
    border-radius: 4px;
    background-color: ${(props: { backgroundColor: string, color: string }) => props.backgroundColor};
    margin-right: 4px;
    -webkit-user-select: none;
    -moz-user-select: none;
    -ms-user-select: none;
    user-select: none;
    cursor: default;
    display: inline;
`
const Label = styled.div`
    width: 200px;
    -webkit-user-select: none;      
    -moz-user-select: none;
    -ms-user-select: none;
    user-select: none;
    cursor: default;
    margin-bottom: 0px;
    padding-top: 1px;
`

const ContractField = (props: { fieldName: string, fieldType?: string, contractIdentifier: string }) => {
    let dispatch = useRootDispatch();
    const activeFieldIdentifier = useRootSelector(selectActiveFieldIdentifier);
    let fieldIdentifier = `${props.contractIdentifier}::${props.fieldName}`;
    let isFieldActive = activeFieldIdentifier !== undefined && activeFieldIdentifier === fieldIdentifier;

    // var
    let backgroundColor = "rgb(255, 248, 197)"
    let color = "rgb(191, 135, 0)"
    if (props.fieldType === "map") {
        // map
        backgroundColor = "rgb(251, 239, 255)"
        color = "rgb(130, 80, 223)"
    } else if  (props.fieldType === "nft") {
        // nft
        backgroundColor = "rgb(255, 239, 247)"
        color = "rgb(191, 57, 137)"
    } else if (props.fieldType === "ft") {
        // ft
        backgroundColor = "rgb(221, 244, 255)"
        color = "rgb(9, 105, 218)"
    }

    function handleClick(event: MouseEvent) {
        event.preventDefault();
        let payload = { field_name: props.fieldName, contract_identifier: props.contractIdentifier };
        dispatch(watchContractField(payload));
        dispatch(activateField(payload));
    }
    
    return (
        <Container isFieldActive={isFieldActive} onClick={handleClick}>
            {props.fieldType ? <Tag backgroundColor={backgroundColor} color={color}>{props.fieldType}</Tag> : ''}
            <Label> {props.fieldName}</Label>
        </Container>
    );
};

export { ContractField };