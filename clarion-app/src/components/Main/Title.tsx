import { Text } from '@primer/react'
import { MouseEvent } from 'react';
import styled from "styled-components"

export const ContainerX = styled.div`

    color: rgb(55, 53, 47);
    font-size: 40px;
    font-weight: 700;
    letter-space: 0.03em;
    margin-top: 0px;
    cursor: default;
`

const Title = (props: { name: String }) => {

    function handleClick(event: MouseEvent) {
        event.preventDefault();
    }
    
    return (
        <ContainerX onClick={handleClick}>
            <Text> {props.name}</Text>
        </ContainerX>
    );
};

export { Title };