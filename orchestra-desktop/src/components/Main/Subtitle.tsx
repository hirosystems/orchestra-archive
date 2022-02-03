

import { Text } from '@primer/react'
import { MouseEvent } from 'react';
import styled from "styled-components"

export const Container = styled.div`
    color: rgba(55, 53, 47, 0.6);
    font-size: 14px;
    font-weight: 500;
    letter-space: 0.03em;
    margin-top: 0px;
    cursor: default;
`

const Subtitle = (props: { name: String }) => {

    return (
        <Container>
            <Text> {props.name}</Text>
        </Container>
    );
};

export { Subtitle };