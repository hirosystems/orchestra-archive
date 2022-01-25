import { Text } from '@primer/react'
import { MouseEvent } from 'react';
import styled from "styled-components"

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

const Wallet = (props: { walletAddress: String }) => {

    function handleClick(event: MouseEvent) {
        event.preventDefault();
        alert(event.currentTarget.tagName); 
    }
    
    return (
        <Container onClick={handleClick}>
            <Text> {props.walletAddress}</Text>
        </Container>
    );
};

export { Wallet };