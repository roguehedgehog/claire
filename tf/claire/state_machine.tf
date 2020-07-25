resource "aws_sfn_state_machine" "claire_acquisition" {
  name       = "claire_acquisition"
  role_arn   = aws_iam_role.claire_state_machine_role.arn
  definition = data.template_file.claire_acquisition_state_machine.rendered

}

data "template_file" "claire_acquisition_state_machine" {
  template = file("state_machines/acquisition.json")

  vars = {
    move_volumes                 = aws_sfn_state_machine.claire_move_volumes.arn
    create_investigation         = aws_lambda_function.create_investigation.arn
    create_evidence_extractor    = aws_lambda_function.create_evidence_extractor.arn
    poll_evidence_extractor      = aws_lambda_function.poll_evidence_extractor.arn
    prepare_memory_volume        = aws_lambda_function.prepare_memory_volume.arn
    capture_memory               = aws_lambda_function.capture_memory.arn
    upload_memory                = aws_lambda_function.upload_memory.arn
    is_command_complete          = aws_lambda_function.is_command_complete.arn
    isolate_instance             = aws_lambda_function.isolate_instance.arn
    snapshot_disks               = aws_lambda_function.snapshot_disks.arn
    terminate_evidence_extractor = aws_lambda_function.terminate_evidence_extractor.arn
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
