import { Text } from '@primer/react'
import styled from "styled-components"

export const Container = styled.div`
    color: rgba(255, 255, 255, 0.8);
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