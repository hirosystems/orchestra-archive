import '../App.css';
import styled from "styled-components";

const MenuContainer = styled.div`
    display: flex;
    justify-content: center;
    background-color: #000000;
    width: 70px;
`

function Menu() {
    return (
        <MenuContainer data-tauri-drag-region>
        </MenuContainer>
    );
}

export default Menu;
