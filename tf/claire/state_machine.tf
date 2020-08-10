resource "aws_sfn_state_machine" "claire_investigation" {
  name       = "claire_investigation"
  role_arn   = aws_iam_role.claire_state_machine_role.arn
  definition = data.template_file.claire_investigation_state_machine.rendered

}

data "template_file" "claire_investigation_state_machine" {
  template = file("state_machines/investigation.json")

  vars = {
    move_volumes                 = aws_sfn_state_machine.claire_move_volumes.arn
    create_volumes               = aws_lambda_function.create_volumes.arn
    create_investigation         = aws_lambda_function.create_investigation.arn
    create_evidence_extractor    = aws_lambda_function.create_evidence_extractor.arn
    poll_evidence_extractor      = aws_lambda_function.poll_evidence_extractor.arn
    prepare_memory_volume        = aws_lambda_function.prepare_memory_volume.arn
    capture_memory               = aws_lambda_function.capture_memory.arn
    memory_analysis              = aws_lambda_function.memory_analysis.arn
    is_command_complete          = aws_lambda_function.is_command_complete.arn
    isolate_instance             = aws_lambda_function.isolate_instance.arn
    snapshot_volumes             = aws_lambda_function.snapshot_volumes.arn
    snapshot_volumes_ready       = aws_lambda_function.snapshot_volumes_ready.arn
    capture_volumes              = aws_lambda_function.capture_volumes.arn
    terminate_evidence_extractor = aws_lambda_function.terminate_evidence_extractor.arn
    instance_isolation_threshold = var.instance_isolation_threshold
  }
}

resource "aws_sfn_state_machine" "claire_move_volumes" {
  name       = "claire_move_volumes"
  role_arn   = aws_iam_role.claire_state_machine_role.arn
  definition = data.template_file.claire_move_volumes_state_machine.rendered
}

data "template_file" "claire_move_volumes_state_machine" {
  template = file("state_machines/move_volumes.json")

  vars = {
    detach_volumes             = aws_lambda_function.detach_volumes.arn
    is_detach_volumes_complete = aws_lambda_function.is_detach_volumes_complete.arn
    attach_volumes             = aws_lambda_function.attach_volumes.arn
    is_attach_volumes_complete = aws_lambda_function.is_attach_volumes_complete.arn
    destroy_volumes            = aws_lambda_function.destroy_volumes.arn
  }
}
