import { Text } from '@primer/react'
import { MouseEvent } from 'react';
import styled from "styled-components"

export const Container = styled.div`
    width: 256px;
    color: rgba(55, 53, 47, 0.4);
    text-transform: uppercase;
    font-size: 11.5px;
    font-weight: 600;
    letter-space: 0.03em;
    margin-top: 24px;
    -webkit-user-select: none;      
    -moz-user-select: none;
    -ms-user-select: none;
    user-select: none;
    cursor: default;
    padding: 4px;
    a:hover { 
        background-color: yellow;
    }

`

const Section = (props: { name: String }) => {

    function handleClick(event: MouseEvent) {
        event.preventDefault();
    }
    
    return (
        <Container onClick={handleClick}>
            {props.name}
        </Container>
    );
};

export { Section };