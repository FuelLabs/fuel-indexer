'use strict';

const e = React.createElement;

const DASHBOARD_EXAMPLE_URL = "http://127.0.0.1:29987/api/graph/fuel_examples";
const REQUEST_OPTIONS = {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ query: "query { transfer { id contract_id receiver amount asset_id }}", params: "b" }),
};

class TransferList extends React.Component {
    constructor(props) {
        super(props);
        this.state = { isFetching: false, transfers: [], assetToNumTransfers: [] };
    }

    render() {
        var header = e('thead', null,
            e('tr', null,
                e('th', null, `ID`),
                e('th', null, `Asset`),
                e('th', null, `Amount`)
            )
        );

        var footer = e('tfoot', null,
            e('tr', null,
                e('th', null, `ID`),
                e('th', null, `Asset`),
                e('th', null, `Amount`)
            )
        );

        var transfer_elems = this.state.transfers.map((transfer) => {
            if (this.state.assetToNumTransfers[transfer.asset_id]) {
                this.state.assetToNumTransfers[transfer.asset_id] += 1;
            } else {
                this.state.assetToNumTransfers[transfer.asset_id] = 1;
            }

            return (
                e('tr', null,
                    e('th', null, `${transfer.id.substring(0, 6)}`),
                    e('td', null, `${transfer.asset_id.substring(0, 2)}`),
                    e('td', null, `${transfer.amount}`)
                )
            );
        });

        var transfer_list = e('tbody', null, transfer_elems);
        return e('div', null, e('div', null, `List of transfers`), e('table', null, header, footer, transfer_list));
    }

    componentDidMount() {
        this.fetchTransfers();
    }

    fetchTransfers() {
        this.setState({ ...this.state, isFetching: true });
        fetch(DASHBOARD_EXAMPLE_URL, REQUEST_OPTIONS)
            .then(response => response.json())
            .then(result => {
                result.sort((a, b) => a.id - b.id);
                this.setState({ ...this.state, transfers: result, isFetching: false }, () => { });
            })
            .catch(e => {
                console.log(e);
                this.setState({ ...this.state, isFetching: false }, () => { });
            })
    }
}

const domContainer = document.querySelector('#transfer-list');
const root = ReactDOM.createRoot(domContainer);
root.render(e(TransferList));