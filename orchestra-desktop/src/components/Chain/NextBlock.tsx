
import styled from "styled-components";
import { Spinner } from "@primer/react";

export const Container = styled.div`
    width: 64px;
    height: 64px;
    background-color: rgba(255, 255, 255, 0.1);
    border-radius: 8px;
`

export const Inner = styled.div`
    color: rgba(255, 255, 255, 0.5);
    text-transform: uppercase;
    font-size: 10px;
    -webkit-user-select: none;      
    -moz-user-select: none;
    -ms-user-select: none;
    user-select: none;
    cursor: default;
    height: 100%;
    width: 100%;
    text-align: center;
    justify-content: center;
    align-items: center;
    display: flex;
`

const NextBlock = (props: { blockHeight: number }) => {
    return (
        <Container>
            <Inner>
                <Spinner size="medium" sx={{ mr: 0 }} />
            </Inner>
        </Container>
    );
};

export { NextBlock };