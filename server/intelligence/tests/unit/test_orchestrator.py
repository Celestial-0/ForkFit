from src.graph.orchestrator import build_graph


def test_build_graph():
    graph = build_graph()
    
    # Assert nodes are registered correctly
    expected_nodes = {
        "planner",
        "safety",
        "nutrition",
        "budget",
        "culture",
        "recipe",
        "calendar",
        "shopping",
        "consensus",
        "judge",
        "visualization",
    }
    
    # Assert nodes exist in the graph object
    assert expected_nodes.issubset(graph.nodes.keys())
