import { Text } from '@primer/react'
import styled from "styled-components"

export const Container = styled.div`
    color: rgb(9, 105, 218);
    font-size: 24px;
    font-weight: 600;
    letter-space: 0.03em;
    margin-top: 24px;
    cursor: default;
`

const ValueLabel = (props: { name: String }) => {

    return (
        <Container>
            <Text> {props.name}</Text>
        </Container>
    );
};

export { ValueLabel };