import { Text } from '@primer/react'
import { MouseEvent } from 'react';
import styled from "styled-components"
import { useRootSelector, useRootDispatch } from "../../hooks/useRootSelector";
import { selectContracts } from "../../states/StateExplorerState";
import { ContractField } from "./ContractField";

export const Container = styled.div`
    color: rgb(55, 53, 47);
    text-transform: uppercase;
    font-size: 11.5px;
    font-weight: 600;
    letter-space: 0.03em;
    margin-top: 16px;
    margin-left: 12px;
    -webkit-user-select: none;      
    -moz-user-select: none;
    -ms-user-select: none;
    user-select: none;
    cursor: default;
`
export const ContractCell = styled.div`
    color: rgb(55, 53, 47);
    text-transform: uppercase;
    font-size: 11.5px;
    font-weight: 600;
    letter-space: 0.03em;
    margin-top: 16px;
    -webkit-user-select: none;      
    -moz-user-select: none;
    -ms-user-select: none;
    user-select: none;
    cursor: default;
`

const Contract = (props: { contractIdentifier: string }) => {

    const contracts = useRootSelector(selectContracts);

    function handleClick(event: MouseEvent) {
        event.preventDefault();
        alert(event.currentTarget.tagName); 
    }

    let index = 0;
    let cells = [];
    for (const v of contracts[props.contractIdentifier].variables) {
        cells.push(
        <ContractField key={index} fieldName={v.name} fieldType="var" contractIdentifier={props.contractIdentifier} />
    )
    index += 1;
    }
    for (const v of contracts[props.contractIdentifier].maps) {
        cells.push(
        <ContractField key={index} fieldName={v.name} fieldType="map" contractIdentifier={props.contractIdentifier} />
    )
    index += 1;
    }
    for (const v of contracts[props.contractIdentifier].fungible_tokens) {
        cells.push(
        <ContractField key={index} fieldName={v.name} fieldType="ft" contractIdentifier={props.contractIdentifier} />
    )
    index += 1;
    }
    for (const v of contracts[props.contractIdentifier].non_fungible_tokens) {
        cells.push(
        <ContractField key={index} fieldName={v.name} fieldType="nft" contractIdentifier={props.contractIdentifier} />
    )
    index += 1;
    }

    let contractName = `${props.contractIdentifier.split('.')[1]}.clar`;
    return (
        <Container>
            <ContractCell onClick={handleClick}>
                <Text> {contractName}</Text>
            </ContractCell>
            {cells}
        </Container>
    );
};

export { Contract };